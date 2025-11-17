//! File I/O operations for session management
//!
//! This module handles reading and writing session files in the .snek/ directory,
//! including active session tracking, session metadata, chat history, and code contexts.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::snapshot::{CodeContext, ContextSnapshot, Limits};

/// Internal struct for active.json
#[derive(Deserialize)]
#[allow(dead_code)]
struct ActiveJson {
    schema: u32,
    id: String,
    path: String,
}

/// Internal struct for session.json
#[derive(Deserialize)]
#[allow(dead_code)]
struct SessionJson {
    schema: u32,
    id: String,
    name: String,
    version: u64,
    limits: Limits,
    updated_at: String,
}

/// Internal struct for code_snippets.json
#[derive(Deserialize)]
#[allow(dead_code)]
struct CodeSnippetsJson {
    schema: u32,
    snippets: Vec<CodeContext>,
}

/// Find the workspace root by looking for .snek/ directory
///
/// If workspace_dir is provided, uses that as the base.
/// Otherwise, walks up the directory tree from current directory.
/// If not found, creates .snek/ in the appropriate directory.
pub fn find_workspace_root(workspace_dir: Option<PathBuf>) -> Result<PathBuf> {
    // If workspace directory is explicitly provided, use it
    if let Some(workspace) = workspace_dir {
        let snek_dir = workspace.join(".snek");

        // If .snek exists, use it
        if snek_dir.exists() && snek_dir.is_dir() {
            return Ok(snek_dir);
        }

        // Create .snek in the provided workspace directory
        std::fs::create_dir_all(&snek_dir)?;
        initialize_default_session(&snek_dir)?;
        return Ok(snek_dir);
    }

    // No workspace provided, use old behavior: walk up from current directory
    let current = std::env::current_dir()?;
    let mut path = current.as_path();

    loop {
        let snek_dir = path.join(".snek");
        if snek_dir.exists() && snek_dir.is_dir() {
            return Ok(snek_dir);
        }

        match path.parent() {
            Some(parent) => path = parent,
            None => {
                // Not found, create in current directory
                let snek_dir = current.join(".snek");
                std::fs::create_dir_all(&snek_dir)?;
                initialize_default_session(&snek_dir)?;
                return Ok(snek_dir);
            }
        }
    }
}

/// Initialize default session structure
///
/// Creates a default session with context folder and code snippets file.
fn initialize_default_session(snek_root: &Path) -> Result<()> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_dir = snek_root.join("sessions").join(&session_id);
    std::fs::create_dir_all(&session_dir)?;

    // Create context/ folder for markdown files
    std::fs::create_dir_all(session_dir.join("context"))?;

    // Create scripts/ folder
    std::fs::create_dir_all(snek_root.join("scripts"))?;

    // Create commands/ folder
    std::fs::create_dir_all(snek_root.join("commands"))?;

    // Write default session.json
    let session = serde_json::json!({
        "schema": 1,
        "id": session_id,
        "name": "default",
        "version": 0,
        "limits": { "max_tokens": 1600 },
        "updated_at": chrono::Utc::now().to_rfc3339()
    });
    std::fs::write(
        session_dir.join("session.json"),
        serde_json::to_string_pretty(&session)?,
    )?;

    // Write empty code_snippets.json
    let snippets = serde_json::json!({
        "schema": 1,
        "snippets": []
    });
    std::fs::write(
        session_dir.join("code_snippets.json"),
        serde_json::to_string_pretty(&snippets)?,
    )?;

    // Write active.json
    let active = serde_json::json!({
        "schema": 1,
        "id": session_id,
        "path": format!("sessions/{}", session_id)
    });
    std::fs::write(
        snek_root.join("active.json"),
        serde_json::to_string_pretty(&active)?,
    )?;

    // Write scripts if they don't exist
    write_script_file(snek_root, "scripts/new-session.sh", include_str!("../templates/scripts/new-session.sh"))?;
    write_script_file(snek_root, "scripts/switch-session.sh", include_str!("../templates/scripts/switch-session.sh"))?;

    // Write commands if they don't exist
    write_script_file(snek_root, "commands/snek.share.md", include_str!("../templates/commands/snek.share.md"))?;

    Ok(())
}

/// Write a script/command file if it doesn't already exist
fn write_script_file(snek_root: &Path, relative_path: &str, content: &str) -> Result<()> {
    let file_path = snek_root.join(relative_path);
    if !file_path.exists() {
        std::fs::write(&file_path, content)?;

        // Make shell scripts executable
        if relative_path.ends_with(".sh") {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&file_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&file_path, perms)?;
            }
        }
    }
    Ok(())
}

/// Read active.json and resolve to session directory path
pub fn resolve_active_session(snek_root: &Path) -> Result<PathBuf> {
    let active_path = snek_root.join("active.json");
    let content = std::fs::read_to_string(&active_path).context("Failed to read active.json")?;
    let active: ActiveJson =
        serde_json::from_str(&content).context("Failed to parse active.json")?;

    Ok(snek_root.join(&active.path))
}

/// Load complete snapshot from session directory
pub fn load_snapshot(session_dir: &Path) -> Result<ContextSnapshot> {
    // Read session.json
    let session_path = session_dir.join("session.json");
    let session_content =
        std::fs::read_to_string(&session_path).context("Failed to read session.json")?;
    let session: SessionJson =
        serde_json::from_str(&session_content).context("Failed to parse session.json")?;

    // Read code_snippets.json (optional, default to empty)
    let snippets_path = session_dir.join("code_snippets.json");
    let code_snippets = if snippets_path.exists() {
        let snippets_content =
            std::fs::read_to_string(&snippets_path).context("Failed to read code_snippets.json")?;
        let snippets: CodeSnippetsJson =
            serde_json::from_str(&snippets_content).context("Failed to parse code_snippets.json")?;
        snippets.snippets
    } else {
        vec![]
    };

    // Build markdown cache from context/ folder
    let mut markdown_cache = std::collections::HashMap::new();
    let context_dir = session_dir.join("context");
    if context_dir.exists() && context_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&context_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            markdown_cache.insert(filename.to_string(), content);
                        }
                    }
                }
            }
        }
    }

    // Build file cache for code snippets
    let mut file_cache = std::collections::HashMap::new();
    for snippet in &code_snippets {
        // Only read each URI once (multiple snippets may reference same file)
        if !file_cache.contains_key(&snippet.uri) {
            if let Ok(uri) = url::Url::parse(&snippet.uri) {
                if let Ok(file_path) = uri.to_file_path() {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        file_cache.insert(snippet.uri.clone(), content);
                    }
                }
            }
        }
    }

    Ok(ContextSnapshot {
        session_id: session.id,
        version: session.version,
        limits: session.limits,
        session_dir: session_dir.to_path_buf(),
        code_snippets,
        markdown_cache,
        file_cache,
    })
}
