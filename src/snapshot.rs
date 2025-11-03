//! Core data structures for the Snek LSP
//!
//! This module defines the in-memory representation of session state,
//! including chat messages, code contexts, and configuration.

use serde::{Deserialize, Serialize};

/// A single message in the chat history
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    /// Message sender role: "system", "user", or "assistant"
    pub role: String,
    /// Message text content
    pub content: String,
}

/// A code snippet from another file for context
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CodeContext {
    /// File URI (file:// scheme)
    pub uri: String,
    /// First line to include (0-indexed)
    pub start_line: u32,
    /// Last line to include (exclusive)
    pub end_line: u32,
    /// Language identifier (e.g., "rust", "python")
    pub language_id: String,
    /// Extracted code content
    pub code: String,
    /// Optional human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Last modification timestamp (RFC3339)
    pub last_modified: String,
}

/// Token limits for model completion
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Limits {
    /// Maximum tokens for model completion
    pub max_tokens: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { max_tokens: 1600 }
    }
}

/// In-memory snapshot of the active session
#[derive(Clone, Debug)]
pub struct ContextSnapshot {
    /// Current session ID
    pub session_id: String,
    /// Session version (for change detection)
    pub version: u64,
    /// Token limits
    pub limits: Limits,
    /// Ordered chat history
    pub chat_messages: Vec<ChatMessage>,
    /// Code snippets from other files
    pub code_contexts: Vec<CodeContext>,
}

impl Default for ContextSnapshot {
    fn default() -> Self {
        Self {
            session_id: "default".to_string(),
            version: 0,
            limits: Limits::default(),
            chat_messages: vec![],
            code_contexts: vec![],
        }
    }
}
