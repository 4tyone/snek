use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CodeContext {
    pub uri: String,
    pub start_line: u32,
    pub end_line: u32,
    pub language_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Limits {
    pub max_tokens: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { max_tokens: 1600 }
    }
}

#[derive(Clone, Debug)]
pub struct ContextSnapshot {
    pub session_id: String,
    pub version: u64,
    pub limits: Limits,
    pub session_dir: PathBuf,
    pub code_snippets: Vec<CodeContext>,
    pub markdown_cache: HashMap<String, String>,
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
