use anyhow::Result;
use snek::lsp::server;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("[SNEK] Starting Snek Language Server...");

    match server::serve_stdio().await {
        Ok(()) => {
            eprintln!("[SNEK] Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            eprintln!("[SNEK] Server error: {}", e);
            Err(e)
        }
    }
}
