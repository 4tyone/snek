//! Core data structures for the Snek LSP
//!
//! This module defines the in-memory representation of session state,
//! including chat messages, code contexts, and configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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
    /// Optional human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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

pub struct Yerevan {}

/// In-memory snapshot of the active session
#[derive(Clone, Debug)]
pub struct ContextSnapshot {
    /// Current session ID
    pub session_id: String,
    /// Session version (for change detection)
    pub version: u64,
    /// Token limits
    pub limits: Limits,
    /// Path to the session directory (for reading context files on-demand)
    pub session_dir: PathBuf,
    /// Code snippets from other files
    pub code_snippets: Vec<CodeContext>,
    /// Cache of markdown files: filename -> content
    pub markdown_cache: HashMap<String, String>,
    /// Cache of code files: URI -> full file content
    pub file_cache: HashMap<String, String>,
}

impl Default for ContextSnapshot {
    fn default() -> Self {
        Self {
            session_id: "default".to_string(),
            version: 0,
            limits: Limits::default(),
            session_dir: PathBuf::new(),
            code_snippets: vec![],
            markdown_cache: HashMap::new(),
            file_cache: HashMap::new(),
        }
    }
}
