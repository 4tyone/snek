//! File system watcher for context updates
//!
//! Monitors session files and context files for changes, updating the
//! HashMap caches incrementally without blocking completion requests.

use anyhow::Result;
use arc_swap::ArcSwap;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::session_io::{load_snapshot, resolve_active_session};
use crate::snapshot::ContextSnapshot;

/// File system watcher for session and context files
pub struct SessionWatcher {
    _handle: tokio::task::JoinHandle<()>,
}

impl SessionWatcher {
    /// Start watching for file system changes
    pub fn start(
        snek_root: PathBuf,
        snapshot: Arc<ArcSwap<ContextSnapshot>>,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);

        // Create file system watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        )?;

        // Get current session directory
        let session_dir = resolve_active_session(&snek_root)?;

        // Watch active.json to detect session switches
        let active_path = snek_root.join("active.json");
        if active_path.exists() {
            watcher.watch(&active_path, RecursiveMode::NonRecursive)?;
            eprintln!("[SNEK] Watching active.json for session changes");
        }

        // Watch code_snippets.json
        let snippets_path = session_dir.join("code_snippets.json");
        if snippets_path.exists() {
            watcher.watch(&snippets_path, RecursiveMode::NonRecursive)?;
        }

        // Watch context/ folder recursively
        let context_dir = session_dir.join("context");
        if context_dir.exists() {
            watcher.watch(&context_dir, RecursiveMode::Recursive)?;
        }

        // Watch individual code files from current snapshot
        let current_snapshot = snapshot.load();
        let mut watched_files: HashSet<PathBuf> = HashSet::new();

        for ctx in &current_snapshot.code_snippets {
            if let Ok(uri) = url::Url::parse(&ctx.uri) {
                if let Ok(file_path) = uri.to_file_path() {
                    if file_path.exists() && watcher.watch(&file_path, RecursiveMode::NonRecursive).is_ok() {
                        watched_files.insert(file_path);
                    }
                }
            }
        }

        // Spawn background watch loop
        let handle = tokio::spawn(async move {
            watch_loop(rx, snek_root, session_dir, snapshot, watcher, watched_files).await;
        });

        Ok(Self { _handle: handle })
    }
}

/// Main watch loop that processes file system events
async fn watch_loop(
    mut rx: mpsc::Receiver<Event>,
    snek_root: PathBuf,
    session_dir: PathBuf,
    snapshot: Arc<ArcSwap<ContextSnapshot>>,
    mut watcher: RecommendedWatcher,
    mut watched_files: HashSet<PathBuf>,
) {
    let debounce_duration = Duration::from_millis(200);
    let mut pending_snippets_reload = false;
    let mut pending_markdown_updates: HashSet<PathBuf> = HashSet::new();
    let mut pending_code_updates: HashSet<PathBuf> = HashSet::new();

    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                for path in &event.paths {
                    // Check if active.json changed
                    if path.file_name() == Some(std::ffi::OsStr::new("active.json"))
                        && path.parent() == Some(snek_root.as_path()) {
                        eprintln!("╔════════════════════════════════════════════════════════════════════╗");
                        eprintln!("║                   🔄 SESSION SWITCHED 🔄                           ║");
                        eprintln!("╠════════════════════════════════════════════════════════════════════╣");
                        eprintln!("║ Active session has changed.                                        ║");
                        eprintln!("║                                                                    ║");
                        eprintln!("║ Please restart the snek LSP server to load the new session:       ║");
                        eprintln!("║   - Different context files                                        ║");
                        eprintln!("║   - Different code snippets                                        ║");
                        eprintln!("║   - Different session limits                                       ║");
                        eprintln!("║                                                                    ║");
                        eprintln!("║ In VS Code: Reload window (Cmd/Ctrl + Shift + P > Reload Window)  ║");
                        eprintln!("╚════════════════════════════════════════════════════════════════════╝");
                    }
                    // Check if code_snippets.json changed
                    else if path.file_name() == Some(std::ffi::OsStr::new("code_snippets.json")) {
                        eprintln!("[SNEK] code_snippets.json changed, scheduling reload");
                        pending_snippets_reload = true;
                    }
                    // Check if markdown file in context/ changed
                    else if path.extension() == Some(std::ffi::OsStr::new("md"))
                        && path.starts_with(&session_dir.join("context")) {
                        eprintln!("[SNEK] Markdown file changed: {:?}", path);
                        pending_markdown_updates.insert(path.clone());
                    }
                    // Check if watched code file changed
                    else if watched_files.contains(path) {
                        eprintln!("[SNEK] Code file changed: {:?}", path);
                        pending_code_updates.insert(path.clone());
                    }
                }
            }
            _ = tokio::time::sleep(debounce_duration) => {
                // Process pending updates after debounce
                if pending_snippets_reload {
                    if let Err(e) = reload_code_snippets(
                        &snek_root,
                        &session_dir,
                        &snapshot,
                        &mut watcher,
                        &mut watched_files,
                    ) {
                        eprintln!("[SNEK] Failed to reload code snippets: {}", e);
                    }
                    pending_snippets_reload = false;
                    pending_code_updates.clear(); // Snippets reload includes code cache refresh
                }

                if !pending_markdown_updates.is_empty() {
                    update_markdown_cache(&session_dir, &snapshot, &pending_markdown_updates);
                    pending_markdown_updates.clear();
                }

                if !pending_code_updates.is_empty() {
                    update_code_cache(&snapshot, &pending_code_updates);
                    pending_code_updates.clear();
                }
            }
        }
    }
}

/// Reload code_snippets.json and update file cache
fn reload_code_snippets(
    _snek_root: &Path,
    session_dir: &Path,
    snapshot: &Arc<ArcSwap<ContextSnapshot>>,
    watcher: &mut RecommendedWatcher,
    watched_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    eprintln!("[SNEK] Reloading code_snippets.json...");

    // Load new snapshot (includes repopulated caches)
    let new_snapshot = load_snapshot(session_dir)?;

    // Update watch list for code files
    let new_files: HashSet<PathBuf> = new_snapshot
        .code_snippets
        .iter()
        .filter_map(|ctx| {
            url::Url::parse(&ctx.uri)
                .ok()
                .and_then(|uri| uri.to_file_path().ok())
        })
        .collect();

    // Unwatch removed files
    for old_file in watched_files.iter() {
        if !new_files.contains(old_file) {
            let _ = watcher.unwatch(old_file);
            eprintln!("[SNEK] Unwatched: {:?}", old_file);
        }
    }

    // Watch new files
    for new_file in &new_files {
        if !watched_files.contains(new_file) && new_file.exists() {
            if watcher.watch(new_file, RecursiveMode::NonRecursive).is_ok() {
                eprintln!("[SNEK] Now watching: {:?}", new_file);
            }
        }
    }

    *watched_files = new_files;
    snapshot.store(Arc::new(new_snapshot));

    eprintln!("[SNEK] Code snippets reloaded successfully");
    Ok(())
}

/// Update markdown cache for changed files
fn update_markdown_cache(
    _session_dir: &Path,
    snapshot: &Arc<ArcSwap<ContextSnapshot>>,
    changed_paths: &HashSet<PathBuf>,
) {
    let current = snapshot.load();
    let mut new_snapshot = (**current).clone();

    for path in changed_paths {
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            let filename_string = filename.to_string();

            // Check if file was deleted or still exists
            if path.exists() {
                // File modified or added
                if let Ok(content) = std::fs::read_to_string(path) {
                    new_snapshot.markdown_cache.insert(filename_string.clone(), content);
                    eprintln!("[SNEK] Updated markdown cache: {}", filename);
                }
            } else {
                // File deleted
                new_snapshot.markdown_cache.remove(&filename_string);
                eprintln!("[SNEK] Removed from markdown cache: {}", filename);
            }
        }
    }

    snapshot.store(Arc::new(new_snapshot));
}

/// Update file cache for changed code files
fn update_code_cache(
    snapshot: &Arc<ArcSwap<ContextSnapshot>>,
    changed_paths: &HashSet<PathBuf>,
) {
    let current = snapshot.load();
    let mut new_snapshot = (**current).clone();

    // Find URIs for changed paths
    for path in changed_paths {
        // Find which snippets reference this path
        for snippet in &current.code_snippets {
            if let Ok(uri) = url::Url::parse(&snippet.uri) {
                if let Ok(snippet_path) = uri.to_file_path() {
                    if snippet_path == *path {
                        // Found a snippet referencing this file
                        if path.exists() {
                            if let Ok(content) = std::fs::read_to_string(path) {
                                new_snapshot.file_cache.insert(snippet.uri.clone(), content);
                                eprintln!("[SNEK] Updated file cache: {}", snippet.uri);
                            }
                        } else {
                            new_snapshot.file_cache.remove(&snippet.uri);
                            eprintln!("[SNEK] Removed from file cache: {}", snippet.uri);
                        }
                        break;
                    }
                }
            }
        }
    }

    snapshot.store(Arc::new(new_snapshot));
}
