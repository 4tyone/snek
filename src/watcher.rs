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

pub struct SessionWatcher {
    _handle: tokio::task::JoinHandle<()>,
}

impl SessionWatcher {
    pub fn start(
        snek_root: PathBuf,
        snapshot: Arc<ArcSwap<ContextSnapshot>>,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        )?;

        let session_dir = resolve_active_session(&snek_root)?;

        let active_path = snek_root.join("active.json");
        if active_path.exists() {
            watcher.watch(&active_path, RecursiveMode::NonRecursive)?;
            eprintln!("[SNEK] Watching active.json for session changes");
        }

        let snippets_path = session_dir.join("code_snippets.json");
        if snippets_path.exists() {
            watcher.watch(&snippets_path, RecursiveMode::NonRecursive)?;
        }

        let context_dir = session_dir.join("context");
        if context_dir.exists() {
            watcher.watch(&context_dir, RecursiveMode::Recursive)?;
        }

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

        let handle = tokio::spawn(async move {
            watch_loop(rx, snek_root, session_dir, snapshot, watcher, watched_files).await;
        });

        Ok(Self { _handle: handle })
    }
}

async fn watch_loop(
    mut rx: mpsc::Receiver<Event>,
    snek_root: PathBuf,
    mut session_dir: PathBuf,
    snapshot: Arc<ArcSwap<ContextSnapshot>>,
    mut watcher: RecommendedWatcher,
    mut watched_files: HashSet<PathBuf>,
) {
    let debounce_duration = Duration::from_millis(200);
    let mut pending_snippets_reload = false;
    let mut pending_markdown_updates: HashSet<PathBuf> = HashSet::new();
    let mut pending_code_updates: HashSet<PathBuf> = HashSet::new();
    let mut pending_session_switch = false;

    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                for path in &event.paths {
                    if path.file_name() == Some(std::ffi::OsStr::new("active.json"))
                        && path.parent() == Some(snek_root.as_path()) {
                        eprintln!("[SNEK] active.json changed, scheduling session switch");
                        pending_session_switch = true;
                    }
                    else if path.file_name() == Some(std::ffi::OsStr::new("code_snippets.json")) {
                        eprintln!("[SNEK] code_snippets.json changed, scheduling reload");
                        pending_snippets_reload = true;
                    }
                    else if path.extension() == Some(std::ffi::OsStr::new("md"))
                        && path.starts_with(&session_dir.join("context")) {
                        eprintln!("[SNEK] Markdown file changed: {:?}", path);
                        pending_markdown_updates.insert(path.clone());
                    }
                    else if watched_files.contains(path) {
                        eprintln!("[SNEK] Code file changed: {:?}", path);
                        pending_code_updates.insert(path.clone());
                    }
                }
            }
            _ = tokio::time::sleep(debounce_duration) => {
                if pending_session_switch {
                    match switch_session(
                        &snek_root,
                        &mut session_dir,
                        &snapshot,
                        &mut watcher,
                        &mut watched_files,
                    ) {
                        Ok(()) => {
                            pending_markdown_updates.clear();
                            pending_code_updates.clear();
                            pending_snippets_reload = false;
                        }
                        Err(e) => {
                            eprintln!("[SNEK] Failed to switch session: {}", e);
                        }
                    }
                    pending_session_switch = false;
                    continue;
                }

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
                    pending_code_updates.clear();
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

fn switch_session(
    snek_root: &Path,
    session_dir: &mut PathBuf,
    snapshot: &Arc<ArcSwap<ContextSnapshot>>,
    watcher: &mut RecommendedWatcher,
    watched_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    eprintln!("[SNEK] Switching session...");

    let new_session_dir = resolve_active_session(snek_root)?;

    if new_session_dir == *session_dir {
        eprintln!("[SNEK] Session unchanged, skipping switch");
        return Ok(());
    }

    eprintln!("[SNEK] New session: {:?}", new_session_dir);

    let old_snippets_path = session_dir.join("code_snippets.json");
    if old_snippets_path.exists() {
        let _ = watcher.unwatch(&old_snippets_path);
    }
    let old_context_dir = session_dir.join("context");
    if old_context_dir.exists() {
        let _ = watcher.unwatch(&old_context_dir);
    }

    for file in watched_files.iter() {
        let _ = watcher.unwatch(file);
    }
    watched_files.clear();

    let new_snapshot = load_snapshot(&new_session_dir)?;

    let new_snippets_path = new_session_dir.join("code_snippets.json");
    if new_snippets_path.exists() {
        watcher.watch(&new_snippets_path, RecursiveMode::NonRecursive)?;
    }
    let new_context_dir = new_session_dir.join("context");
    if new_context_dir.exists() {
        watcher.watch(&new_context_dir, RecursiveMode::Recursive)?;
    }

    for snippet in &new_snapshot.code_snippets {
        if let Ok(uri) = url::Url::parse(&snippet.uri) {
            if let Ok(file_path) = uri.to_file_path() {
                if file_path.exists() && watcher.watch(&file_path, RecursiveMode::NonRecursive).is_ok() {
                    watched_files.insert(file_path);
                }
            }
        }
    }

    *session_dir = new_session_dir;
    snapshot.store(Arc::new(new_snapshot));

    eprintln!("[SNEK] Session switched successfully!");
    Ok(())
}

fn reload_code_snippets(
    _snek_root: &Path,
    session_dir: &Path,
    snapshot: &Arc<ArcSwap<ContextSnapshot>>,
    watcher: &mut RecommendedWatcher,
    watched_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    eprintln!("[SNEK] Reloading code_snippets.json...");

    let new_snapshot = load_snapshot(session_dir)?;

    let new_files: HashSet<PathBuf> = new_snapshot
        .code_snippets
        .iter()
        .filter_map(|ctx| {
            url::Url::parse(&ctx.uri)
                .ok()
                .and_then(|uri| uri.to_file_path().ok())
        })
        .collect();

    for old_file in watched_files.iter() {
        if !new_files.contains(old_file) {
            let _ = watcher.unwatch(old_file);
            eprintln!("[SNEK] Unwatched: {:?}", old_file);
        }
    }

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

            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    new_snapshot.markdown_cache.insert(filename_string.clone(), content);
                    eprintln!("[SNEK] Updated markdown cache: {}", filename);
                }
            } else {
                new_snapshot.markdown_cache.remove(&filename_string);
                eprintln!("[SNEK] Removed from markdown cache: {}", filename);
            }
        }
    }

    snapshot.store(Arc::new(new_snapshot));
}

fn update_code_cache(
    snapshot: &Arc<ArcSwap<ContextSnapshot>>,
    changed_paths: &HashSet<PathBuf>,
) {
    let current = snapshot.load();
    let mut new_snapshot = (**current).clone();

    for path in changed_paths {
        for snippet in &current.code_snippets {
            if let Ok(uri) = url::Url::parse(&snippet.uri) {
                if let Ok(snippet_path) = uri.to_file_path() {
                    if snippet_path == *path {
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
