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
/// Walks up the directory tree from current directory.
/// If not found, creates .snek/ in current directory.
pub fn find_workspace_root() -> Result<PathBuf> {
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

    // Read all markdown files from context/ folder
    let context_dir = session_dir.join("context");
    let mut markdown_context = String::new();
    
    if context_dir.exists() && context_dir.is_dir() {
        let mut context_files = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&context_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                    context_files.push(path);
                }
            }
        }
        
        // Sort files alphabetically for consistent ordering
        context_files.sort();
        
        // Combine all markdown files with filenames
        for path in context_files {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if !markdown_context.is_empty() {
                    markdown_context.push_str("\n\n---\n\n");
                }
                // Add filename header
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    markdown_context.push_str(&format!("## {}\n\n", filename));
                }
                markdown_context.push_str(&content);
            }
        }
    }

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

    Ok(ContextSnapshot {
        session_id: session.id,
        version: session.version,
        limits: session.limits,
        markdown_context,
        code_snippets,
    })
}

/// Update a specific code context by reading from the actual file
pub fn update_context_from_file(context: &mut CodeContext) -> Result<()> {
    let uri = url::Url::parse(&context.uri).context("Invalid URI in context")?;

    let file_path = uri
        .to_file_path()
        .map_err(|_| anyhow::anyhow!("Cannot convert URI to file path"))?;

    let content = std::fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;

    let lines: Vec<&str> = content.lines().collect();
    let start = context.start_line as usize;
    let end = (context.end_line as usize).min(lines.len());

    if start >= lines.len() {
        anyhow::bail!("Start line {} exceeds file length {}", start, lines.len());
    }

    let extracted_lines = &lines[start..end];
    context.code = extracted_lines.join("\n");
    context.last_modified = chrono::Utc::now().to_rfc3339();

    Ok(())
}
