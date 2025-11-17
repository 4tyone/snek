//! LSP server setup and initialization
//!
//! This module handles server startup, workspace initialization,
//! and custom method registration.

use std::sync::Arc;
use tokio::sync::RwLock;

use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use tower_lsp::{LspService, Server};

use crate::document_store::DocumentStore;
use crate::lsp::backend::{Backend, InlineCompletionParams};
use crate::model::ModelClient;
use crate::session_io::{find_workspace_root, load_snapshot, resolve_active_session};
use crate::watcher::SessionWatcher;

/// Start the LSP server on stdio
pub async fn serve_stdio(workspace_dir: Option<std::path::PathBuf>) -> Result<()> {
    eprintln!("[SNEK] Initializing workspace...");

    // Find or create .snek/ directory
    let snek_root = find_workspace_root(workspace_dir).context("Failed to find or create .snek/ directory")?;
    eprintln!("[SNEK] Workspace root: {:?}", snek_root);

    // Load active session
    let session_dir =
        resolve_active_session(&snek_root).context("Failed to resolve active session")?;
    eprintln!("[SNEK] Active session: {:?}", session_dir);

    // Load initial snapshot
    let snapshot = load_snapshot(&session_dir).context("Failed to load session snapshot")?;
    eprintln!(
        "[SNEK] Loaded session: {} (version {})",
        snapshot.session_id, snapshot.version
    );

    // Create shared snapshot with ArcSwap
    let snapshot_arc = Arc::new(ArcSwap::from_pointee(snapshot));

    // Start file watcher for incremental cache updates
    eprintln!("[SNEK] Starting file watcher...");
    let _watcher = SessionWatcher::start(snek_root.clone(), snapshot_arc.clone())?;

    // Initialize empty API key - will be loaded from VSCode settings after initialization
    let api_key = Arc::new(RwLock::new(String::new()));

    // Hardcoded API URL and model name
    let api_url = "https://openai-proxy-aifp.onrender.com/v1/chat/completions".to_string();
    let model_name = "glm-4.6".to_string();

    eprintln!("[SNEK] API key will be loaded from VSCode settings after initialization");

    // Create model client (API key will be provided at request time)
    let model = Arc::new(ModelClient::new(api_url, model_name));

    // Create document store
    let documents = Arc::new(DocumentStore::new());

    // Set up LSP service
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| {
        Backend::new(
            client,
            snapshot_arc.clone(),
            documents.clone(),
            model.clone(),
            api_key.clone(),
        )
    })
    .custom_method(
        "snek/inline",
        |backend: &Backend, params: InlineCompletionParams| {
            let backend = backend.clone();
            async move { backend.handle_inline_completion(params).await }
        },
    )
    .finish();

    eprintln!("[SNEK] Server ready, listening on stdio...");
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

// Helper trait to enable cloning Backend in custom_method closure
impl Clone for Backend {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            snapshot: self.snapshot.clone(),
            documents: self.documents.clone(),
            model: self.model.clone(),
            api_key: self.api_key.clone(),
        }
    }
}
