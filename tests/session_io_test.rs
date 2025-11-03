//! Integration tests for session_io module

use anyhow::Result;
use snek::session_io::{load_snapshot, resolve_active_session, update_context_from_file};
use snek::snapshot::CodeContext;
use std::path::Path;
use tempfile::TempDir;

/// Helper to create a test session structure
fn create_test_session(temp_dir: &Path) -> Result<()> {
    let session_id = "test-session-123";
    let session_dir = temp_dir.join("sessions").join(session_id);
    std::fs::create_dir_all(&session_dir)?;

    // Write session.json
    let session = serde_json::json!({
        "schema": 1,
        "id": session_id,
        "name": "test",
        "version": 42,
        "limits": { "max_tokens": 2000 },
        "updated_at": "2025-11-03T00:00:00Z"
    });
    std::fs::write(
        session_dir.join("session.json"),
        serde_json::to_string_pretty(&session)?,
    )?;

    // Write chat.json
    let chat = serde_json::json!({
        "schema": 1,
        "messages": [
            { "role": "system", "content": "You are a helpful assistant" },
            { "role": "user", "content": "Hello" }
        ]
    });
    std::fs::write(
        session_dir.join("chat.json"),
        serde_json::to_string_pretty(&chat)?,
    )?;

    // Write context.json
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
        temp_dir.join("active.json"),
        serde_json::to_string_pretty(&active)?,
    )?;

    Ok(())
}

#[test]
fn test_resolve_active_session() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_test_session(temp_dir.path())?;

    let session_dir = resolve_active_session(temp_dir.path())?;
    assert!(session_dir.ends_with("sessions/test-session-123"));
    assert!(session_dir.exists());

    Ok(())
}

#[test]
fn test_load_snapshot() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_test_session(temp_dir.path())?;

    let session_dir = temp_dir.path().join("sessions/test-session-123");
    let snapshot = load_snapshot(&session_dir)?;

    assert_eq!(snapshot.session_id, "test-session-123");
    assert_eq!(snapshot.version, 42);
    assert_eq!(snapshot.limits.max_tokens, 2000);
    assert_eq!(snapshot.chat_messages.len(), 2);
    assert_eq!(snapshot.chat_messages[0].role, "system");
    assert_eq!(snapshot.code_contexts.len(), 0);

    Ok(())
}

#[test]
fn test_update_context_from_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a test file
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, "line 0\nline 1\nline 2\nline 3\nline 4\n")?;

    // Create a context pointing to lines 1-3
    let uri = format!("file://{}", test_file.display());
    let mut context = CodeContext {
        uri,
        start_line: 1,
        end_line: 3,
        language_id: "rust".to_string(),
        code: String::new(),
        description: None,
        last_modified: String::new(),
    };

    update_context_from_file(&mut context)?;

    assert_eq!(context.code, "line 1\nline 2");
    assert!(!context.last_modified.is_empty());

    Ok(())
}

#[test]
fn test_update_context_invalid_range() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a test file with only 2 lines
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, "line 0\nline 1\n")?;

    let uri = format!("file://{}", test_file.display());
    let mut context = CodeContext {
        uri,
        start_line: 10, // Beyond file length
        end_line: 20,
        language_id: "rust".to_string(),
        code: String::new(),
        description: None,
        last_modified: String::new(),
    };

    let result = update_context_from_file(&mut context);
    assert!(result.is_err());

    Ok(())
}
