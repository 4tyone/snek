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

pub async fn serve_stdio(workspace_dir: Option<std::path::PathBuf>) -> Result<()> {
    eprintln!("[SNEK] Initializing workspace...");

    let snek_root = find_workspace_root(workspace_dir).context("Failed to find or create .snek/ directory")?;
    eprintln!("[SNEK] Workspace root: {:?}", snek_root);

    let session_dir =
        resolve_active_session(&snek_root).context("Failed to resolve active session")?;
    eprintln!("[SNEK] Active session: {:?}", session_dir);

    let snapshot = load_snapshot(&session_dir).context("Failed to load session snapshot")?;
    eprintln!(
        "[SNEK] Loaded session: {} (version {})",
        snapshot.session_id, snapshot.version
    );

    let snapshot_arc = Arc::new(ArcSwap::from_pointee(snapshot));

    eprintln!("[SNEK] Starting file watcher...");
    let _watcher = SessionWatcher::start(snek_root.clone(), snapshot_arc.clone())?;

    let api_key = Arc::new(RwLock::new(String::new()));
    let api_url = "https://api.cerebras.ai/v1/chat/completions".to_string();
    let model_name = "qwen-3-235b-a22b-instruct-2507".to_string();

    eprintln!("[SNEK] Using Cerebras API: {}", api_url);
    eprintln!("[SNEK] Default model: {}", model_name);
    eprintln!("[SNEK] API key will be loaded from VSCode settings after initialization");

    let model = Arc::new(ModelClient::new(api_url, model_name));
    let documents = Arc::new(DocumentStore::new());

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
