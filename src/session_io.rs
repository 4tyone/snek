//! File I/O operations for session management
//!
//! This module handles reading and writing session files in the .snek/ directory,
//! including active session tracking, session metadata, chat history, and code contexts.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::snapshot::{ChatMessage, CodeContext, ContextSnapshot, Limits};

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

/// Internal struct for chat.json
#[derive(Deserialize)]
#[allow(dead_code)]
struct ChatJson {
    schema: u32,
    messages: Vec<ChatMessage>,
}

/// Internal struct for context.json
#[derive(Deserialize)]
#[allow(dead_code)]
struct ContextJson {
    schema: u32,
    contexts: Vec<CodeContext>,
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
/// Creates a default session with empty chat and context.
fn initialize_default_session(snek_root: &Path) -> Result<()> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_dir = snek_root.join("sessions").join(&session_id);
    std::fs::create_dir_all(&session_dir)?;

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

    // Write empty chat.json
    let chat = serde_json::json!({
        "schema": 1,
        "messages": []
    });
    std::fs::write(
        session_dir.join("chat.json"),
        serde_json::to_string_pretty(&chat)?,
    )?;

    // Write empty context.json
    let context = serde_json::json!({
        "schema": 1,
        "contexts": []
    });
    std::fs::write(
        session_dir.join("context.json"),
        serde_json::to_string_pretty(&context)?,
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

    // Read chat.json (optional, default to empty)
    let chat_path = session_dir.join("chat.json");
    let messages = if chat_path.exists() {
        let chat_content =
            std::fs::read_to_string(&chat_path).context("Failed to read chat.json")?;
        let chat: ChatJson =
            serde_json::from_str(&chat_content).context("Failed to parse chat.json")?;
        chat.messages
    } else {
        vec![]
    };

    // Read context.json (optional, default to empty)
    let context_path = session_dir.join("context.json");
    let code_contexts = if context_path.exists() {
        let context_content =
            std::fs::read_to_string(&context_path).context("Failed to read context.json")?;
        let context: ContextJson =
            serde_json::from_str(&context_content).context("Failed to parse context.json")?;
        context.contexts
    } else {
        vec![]
    };

    Ok(ContextSnapshot {
        session_id: session.id,
        version: session.version,
        limits: session.limits,
        chat_messages: messages,
        code_contexts,
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
