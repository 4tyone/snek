use anyhow::Result;
use snek::lsp::server;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("[SNEK] Starting Snek Language Server...");

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let workspace_dir = parse_workspace_dir(&args);

    if let Some(ref dir) = workspace_dir {
        eprintln!("[SNEK] Workspace directory provided: {}", dir.display());
    } else {
        eprintln!("[SNEK] No workspace directory provided, will search from current directory");
    }

    match server::serve_stdio(workspace_dir).await {
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



/// Parse workspace directory from command-line arguments
/// Supports: --workspace-dir=/path or --workspace-dir /path
fn parse_workspace_dir(args: &[String]) -> Option<PathBuf> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--workspace-dir" || arg == "--workspace" {
            // Next argument is the path
            if let Some(path) = args.get(i + 1) {
                return Some(PathBuf::from(path));
            }
        } else if arg.starts_with("--workspace-dir=") {
            // Format: --workspace-dir=/path
            let path = arg.strip_prefix("--workspace-dir=").unwrap();
            return Some(PathBuf::from(path));
        } else if arg.starts_with("--workspace=") {
            // Format: --workspace=/path
            let path = arg.strip_prefix("--workspace=").unwrap();
            return Some(PathBuf::from(path));
        }
    }
    None
}
