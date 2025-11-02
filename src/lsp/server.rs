use anyhow::Result;
use tower_lsp::{LspService, Server};

use super::backend::Backend;

/// Launch the LSP server bound to stdio.
///
/// Call this from `main` so `cargo run` starts the language server by default.
pub async fn serve_stdio() -> Result<()> {
    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
    Ok(())
}
