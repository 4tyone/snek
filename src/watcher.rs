//! File system watcher for session updates
//!
//! This module watches for changes to session files and context files,
//! automatically reloading the snapshot when changes are detected.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::session_io::{load_snapshot, resolve_active_session, update_context_from_file};
use crate::snapshot::ContextSnapshot;

/// File watcher for session state
pub struct SessionWatcher {
    // Keep watcher alive by storing it
    // The actual watching happens in the background task
}

impl SessionWatcher {
    /// Start watching session files
    pub fn start(snek_root: PathBuf, snapshot: Arc<ArcSwap<ContextSnapshot>>) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);

        // Create watcher
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            })?;

        // Watch active.json
        let active_path = snek_root.join("active.json");
        watcher
            .watch(&active_path, RecursiveMode::NonRecursive)
            .context("Failed to watch active.json")?;

        // Watch session directory (recursively to catch context/ folder changes)
        let session_dir = resolve_active_session(&snek_root)?;
        watcher
            .watch(&session_dir, RecursiveMode::Recursive)
            .context("Failed to watch session directory")?;

        // Get initial context files to watch
        let initial_snapshot = snapshot.load();
        let mut watched_contexts: HashSet<PathBuf> = HashSet::new();

        for ctx in &initial_snapshot.code_snippets {
            if let Ok(uri) = url::Url::parse(&ctx.uri)
                && let Ok(path) = uri.to_file_path()
                    && watcher.watch(&path, RecursiveMode::NonRecursive).is_ok() {
                        watched_contexts.insert(path);
                    }
        }

        // Spawn background task to handle events
        // The watcher is moved into the task and kept alive there
        tokio::spawn(async move {
            watch_loop(rx, snek_root, snapshot, watcher, watched_contexts).await;
        });

        Ok(Self {})
    }
}

/// Main watch loop with debouncing
async fn watch_loop(
    mut rx: mpsc::Receiver<notify::Event>,
    snek_root: PathBuf,
    snapshot: Arc<ArcSwap<ContextSnapshot>>,
    mut watcher: RecommendedWatcher,
    mut watched_contexts: HashSet<PathBuf>,
) {
    let debounce_duration = Duration::from_millis(200);
    let mut pending_reload = false;
    let mut pending_context_updates: HashSet<PathBuf> = HashSet::new();

    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                // Check if this is a session file, markdown context file, or code snippets file
                let is_session_file = event.paths.iter().any(|p| {
                    p.ends_with("active.json") ||
                    p.ends_with("session.json") ||
                    p.ends_with("code_snippets.json") ||
                    (p.extension().and_then(|s| s.to_str()) == Some("md") && 
                     p.parent().and_then(|parent| parent.file_name()).and_then(|name| name.to_str()) == Some("context"))
                });

                if is_session_file {
                    pending_reload = true;
                } else {
                    // Check if it's a watched context file
                    for path in &event.paths {
                        if watched_contexts.contains(path) {
                            pending_context_updates.insert(path.clone());
                        }
                    }
                }
            }
            _ = tokio::time::sleep(debounce_duration) => {
                if pending_reload {
                    if let Err(e) = reload_snapshot(&snek_root, &snapshot, &mut watcher, &mut watched_contexts) {
                        eprintln!("[SNEK] Failed to reload snapshot: {}", e);
                    }
                    pending_reload = false;
                    pending_context_updates.clear();
                } else if !pending_context_updates.is_empty() {
                    if let Err(e) = update_contexts(&snapshot, &pending_context_updates) {
                        eprintln!("[SNEK] Failed to update contexts: {}", e);
                    }
                    pending_context_updates.clear();
                }
            }
        }
    }
}

/// Reload the entire snapshot from disk
fn reload_snapshot(
    snek_root: &Path,
    snapshot: &Arc<ArcSwap<ContextSnapshot>>,
    watcher: &mut RecommendedWatcher,
    watched_contexts: &mut HashSet<PathBuf>,
) -> Result<()> {
    eprintln!("[SNEK] Reloading snapshot...");

    let session_dir = resolve_active_session(snek_root)?;
    let new_snapshot = load_snapshot(&session_dir)?;

    eprintln!(
        "[SNEK] Loaded session {} (version {})",
        new_snapshot.session_id, new_snapshot.version
    );

    // Update watched context files
    let new_contexts: HashSet<PathBuf> = new_snapshot
        .code_snippets
        .iter()
        .filter_map(|ctx| {
            url::Url::parse(&ctx.uri)
                .ok()
                .and_then(|uri| uri.to_file_path().ok())
        })
        .collect();

    // Unwatch removed contexts
    for old_path in watched_contexts.iter() {
        if !new_contexts.contains(old_path) {
            let _ = watcher.unwatch(old_path);
        }
    }

    // Watch new contexts
    for new_path in &new_contexts {
        if !watched_contexts.contains(new_path) {
            let _ = watcher.watch(new_path, RecursiveMode::NonRecursive);
        }
    }

    *watched_contexts = new_contexts;

    // Swap in new snapshot
    snapshot.store(Arc::new(new_snapshot));

    Ok(())
}

/// Update specific context files without full reload
fn update_contexts(
    snapshot: &Arc<ArcSwap<ContextSnapshot>>,
    changed_paths: &HashSet<PathBuf>,
) -> Result<()> {
    eprintln!("[SNEK] Updating {} context files...", changed_paths.len());

    // Load current snapshot
    let current = snapshot.load();
    let mut new_snapshot = (**current).clone();

    // Update each changed context
    for ctx in &mut new_snapshot.code_snippets {
        if let Ok(uri) = url::Url::parse(&ctx.uri)
            && let Ok(path) = uri.to_file_path()
                && changed_paths.contains(&path)
                    && let Err(e) = update_context_from_file(ctx) {
                        eprintln!("[SNEK] Failed to update context {}: {}", ctx.uri, e);
                    }
    }

    // Swap in updated snapshot
    snapshot.store(Arc::new(new_snapshot));

    Ok(())
}
