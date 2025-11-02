use anyhow::Result;
use snek::lsp::server;

#[tokio::main]
async fn main() -> Result<()> {
    server::serve_stdio().await
}
