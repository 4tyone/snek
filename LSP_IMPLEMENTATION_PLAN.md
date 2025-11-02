# Snek LSP Implementation Plan

## Overview

This document outlines the detailed implementation plan for the Snek Language Server Protocol (LSP) component of the Snek ecosystem. The LSP serves as the core intelligence layer that provides AI-powered inline code completions by maintaining a synchronized view of session context, chat history, and code contexts through filesystem-based communication.

### Key Features

1. **Chat History Integration**: Team coding conventions and style guides stored in `chat.json`
2. **Code Context Management**: Reference code from other files via `context.json` with automatic file watching
3. **Incremental Context Updates**: Changes to context files trigger efficient updates without full reload
4. **Filesystem-based Architecture**: No IPC required - all communication via atomic file operations
5. **RAM-based Hot Path**: Zero disk I/O during completion requests (contexts pre-loaded in memory)
6. **OpenAI-compatible API**: Works with any OpenAI-compatible endpoint

## Architecture Principles

1. **Filesystem-based communication**: No direct IPC between CLI, Extension, and LSP
2. **RAM-based hot path**: All completion requests use in-memory snapshots (no disk I/O per completion)
3. **File watching with debouncing**: React to external changes with 150-250ms debounce
4. **Atomic writes**: All file operations use tmp-file-then-rename pattern
5. **Version-based synchronization**: Track changes via monotonic version counter

## File Structure Contract

The LSP reads and watches these files in the workspace:

```
.snek/
  active.json              # Points to current session
  sessions/
    <session-id>/
      session.json         # Metadata + version counter
      chat.json            # Chat history for context
      context.json         # Code context from other files
```

### File Schemas

**active.json**
```json
{
  "schema": 1,
  "id": "b4ubg485",
  "path": "sessions/b4ubg485"
}
```

**session.json**
```json
{
  "schema": 1,
  "id": "b4ubg485",
  "name": "default",
  "version": 42,
  "limits": { "max_tokens": 1600 },
  "updated_at": "2025-10-31T17:22:11Z"
}
```

**chat.json**
```json
{
  "schema": 1,
  "messages": [
    { "role": "system", "content": "You are Snek; write concise code." },
    { "role": "user", "content": "We use Actix; prefer snake_case." },
    { "role": "assistant", "content": "Noted. Avoid async_trait." }
  ]
}
```

**context.json**
```json
{
  "schema": 1,
  "contexts": [
    {
      "uri": "file:///path/to/project/src/utils.rs",
      "start_line": 0,
      "end_line": 120,
      "language_id": "rust",
      "code": "use std::collections::HashMap;\n\npub fn parse_config() -> HashMap<String, String> {\n    // ... implementation\n}",
      "description": "Utility functions for config parsing",
      "last_modified": "2025-11-02T10:30:00Z"
    },
    {
      "uri": "file:///path/to/project/src/models.rs",
      "start_line": 45,
      "end_line": 89,
      "language_id": "rust",
      "code": "pub struct User {\n    pub id: String,\n    pub name: String,\n}",
      "description": "User model definition",
      "last_modified": "2025-11-02T09:15:00Z"
    }
  ]
}
```

> **Context Management**: The LSP watches all files referenced in `context.json` and automatically updates the cached code when those files change. Context is appended to the final prompt (after chat history) to provide relevant code examples without polluting the chat history.

## Implementation Components

### 1. Core Data Structures

**Location**: `src/snapshot.rs` (new file)

Define the in-memory representation of session state:

```rust
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ChatMessage {
    pub role: String,      // "system", "user", "assistant"
    pub content: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CodeContext {
    pub uri: String,
    pub start_line: u32,
    pub end_line: u32,
    pub language_id: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub last_modified: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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
    pub chat_messages: Vec<ChatMessage>,
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
```

**Key Design Decisions**:
- `ContextSnapshot` is `Clone` for cheap Arc-based sharing via `arc-swap`
- Default implementation provides safe fallback during initialization
- Separate `Limits` struct allows future extension (rate limits, context window, etc.)
- `CodeContext` stores complete file snapshots with line ranges for precise context
- `code_contexts` field holds all referenced code snippets that will be watched for changes

### 2. File I/O and Parsing

**Location**: `src/session_io.rs` (new file)

Handle all disk operations for reading session files:

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::snapshot::{ChatMessage, CodeContext, ContextSnapshot, Limits};

#[derive(Deserialize)]
struct ActiveJson {
    schema: u32,
    id: String,
    path: String,
}

#[derive(Deserialize)]
struct SessionJson {
    schema: u32,
    id: String,
    name: String,
    version: u64,
    limits: Limits,
    updated_at: String,
}

#[derive(Deserialize)]
struct ChatJson {
    schema: u32,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ContextJson {
    schema: u32,
    contexts: Vec<CodeContext>,
}

/// Find the workspace root by looking for .snek/ directory
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
fn initialize_default_session(snek_root: &Path) -> Result<()> {
    let session_id = "UUID"; // generates a random id with UUID
    let session_dir = snek_root.join("sessions").join(session_id);
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
        serde_json::to_string_pretty(&session)?
    )?;
    
    // Write empty chat.json
    let chat = serde_json::json!({
        "schema": 1,
        "messages": []
    });
    std::fs::write(
        session_dir.join("chat.json"),
        serde_json::to_string_pretty(&chat)?
    )?;
    
    // Write empty context.json
    let context = serde_json::json!({
        "schema": 1,
        "contexts": []
    });
    std::fs::write(
        session_dir.join("context.json"),
        serde_json::to_string_pretty(&context)?
    )?;
    
    // Write active.json
    let active = serde_json::json!({
        "schema": 1,
        "id": session_id,
        "path": format!("sessions/{}", session_id)
    });
    std::fs::write(
        snek_root.join("active.json"),
        serde_json::to_string_pretty(&active)?
    )?;
    
    Ok(())
}

/// Read active.json and resolve to session directory path
pub fn resolve_active_session(snek_root: &Path) -> Result<PathBuf> {
    let active_path = snek_root.join("active.json");
    let content = std::fs::read_to_string(&active_path)
        .context("Failed to read active.json")?;
    let active: ActiveJson = serde_json::from_str(&content)
        .context("Failed to parse active.json")?;
    
    Ok(snek_root.join(&active.path))
}

/// Load complete snapshot from session directory
pub fn load_snapshot(session_dir: &Path) -> Result<ContextSnapshot> {
    // Read session.json
    let session_path = session_dir.join("session.json");
    let session_content = std::fs::read_to_string(&session_path)
        .context("Failed to read session.json")?;
    let session: SessionJson = serde_json::from_str(&session_content)
        .context("Failed to parse session.json")?;
    
    // Read chat.json (optional, default to empty)
    let chat_path = session_dir.join("chat.json");
    let messages = if chat_path.exists() {
        let chat_content = std::fs::read_to_string(&chat_path)
            .context("Failed to read chat.json")?;
        let chat: ChatJson = serde_json::from_str(&chat_content)
            .context("Failed to parse chat.json")?;
        chat.messages
    } else {
        vec![]
    };
    
    // Read context.json (optional, default to empty)
    let context_path = session_dir.join("context.json");
    let code_contexts = if context_path.exists() {
        let context_content = std::fs::read_to_string(&context_path)
            .context("Failed to read context.json")?;
        let context: ContextJson = serde_json::from_str(&context_content)
            .context("Failed to parse context.json")?;
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
    let uri = url::Url::parse(&context.uri)
        .context("Invalid URI in context")?;
    
    let file_path = uri.to_file_path()
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
```

**Key Design Decisions**:
- `find_workspace_root()` walks up directory tree, creates `.snek/` if not found
- `initialize_default_session()` creates minimal valid structure including `context.json`
- Graceful handling of missing `chat.json` and `context.json` (defaults to empty arrays)
- `update_context_from_file()` extracts specified line ranges from actual files
- All errors use `anyhow::Context` for clear error messages
- Context updates preserve metadata (URI, line ranges, description) while refreshing code

**Dependencies to add**:
- `chrono = "0.4"` for timestamp generation
- `url = "2"` for URI parsing and file path conversion

### 3. File Watching System

**Location**: `src/watcher.rs` (new file)

Implement debounced file watching with automatic session switching and context file monitoring:

```rust
use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use arc_swap::ArcSwap;
use crate::snapshot::ContextSnapshot;
use crate::session_io::{resolve_active_session, load_snapshot, update_context_from_file};

pub struct SessionWatcher {
    _watcher: RecommendedWatcher,
}

impl SessionWatcher {
    pub fn start(
        snek_root: PathBuf,
        snapshot: Arc<ArcSwap<ContextSnapshot>>,
    ) -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
        
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        
        // Watch active.json
        let active_path = snek_root.join("active.json");
        watcher.watch(&active_path, RecursiveMode::NonRecursive)?;
        
        // Load initial session
        let initial_session_dir = resolve_active_session(&snek_root)?;
        let initial_snapshot = load_snapshot(&initial_session_dir)?;
        
        // Watch initial session files
        let session_json = initial_session_dir.join("session.json");
        let chat_json = initial_session_dir.join("chat.json");
        let context_json = initial_session_dir.join("context.json");
        watcher.watch(&session_json, RecursiveMode::NonRecursive)?;
        watcher.watch(&chat_json, RecursiveMode::NonRecursive)?;
        watcher.watch(&context_json, RecursiveMode::NonRecursive)?;
        
        // Watch all context files
        let mut watched_context_files = HashSet::new();
        for context in &initial_snapshot.code_contexts {
            if let Ok(uri) = url::Url::parse(&context.uri) {
                if let Ok(path) = uri.to_file_path() {
                    if watcher.watch(&path, RecursiveMode::NonRecursive).is_ok() {
                        watched_context_files.insert(path);
                    }
                }
            }
        }
        
        snapshot.store(Arc::new(initial_snapshot));
        
        // Spawn debounce thread
        let snapshot_clone = snapshot.clone();
        let snek_root_clone = snek_root.clone();
        std::thread::spawn(move || {
            Self::watch_loop(rx, snek_root_clone, snapshot_clone, watched_context_files);
        });
        
        Ok(Self { _watcher: watcher })
    }
    
    fn watch_loop(
        rx: std::sync::mpsc::Receiver<notify::Result<Event>>,
        snek_root: PathBuf,
        snapshot: Arc<ArcSwap<ContextSnapshot>>,
        mut watched_context_files: HashSet<PathBuf>,
    ) {
        let mut last_reload = Instant::now();
        let debounce_duration = Duration::from_millis(200);
        let mut pending_reload = false;
        let mut pending_context_update = false;
        let mut modified_context_files = HashSet::new();
        
        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(event)) => {
                    // Check if this is a context file modification
                    if let Some(path) = event.paths.first() {
                        if watched_context_files.contains(path) {
                            pending_context_update = true;
                            modified_context_files.insert(path.clone());
                        } else {
                            // Session file changed
                            pending_reload = true;
                        }
                    }
                }
                Ok(Err(_)) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Check if we should reload or update contexts
                    if last_reload.elapsed() >= debounce_duration {
                        if pending_reload {
                            // Full reload (session/chat/context.json changed)
                            if let Err(e) = Self::reload_snapshot(
                                &snek_root,
                                &snapshot,
                                &mut watched_context_files
                            ) {
                                eprintln!("[SNEK] Failed to reload snapshot: {}", e);
                            }
                            pending_reload = false;
                            pending_context_update = false;
                            modified_context_files.clear();
                        } else if pending_context_update {
                            // Incremental context update (context files changed)
                            Self::update_contexts(
                                &snapshot,
                                &modified_context_files
                            );
                            pending_context_update = false;
                            modified_context_files.clear();
                        }
                        last_reload = Instant::now();
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }
    
    fn reload_snapshot(
        snek_root: &Path,
        snapshot: &Arc<ArcSwap<ContextSnapshot>>,
        watched_context_files: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        let session_dir = resolve_active_session(snek_root)?;
        let new_snapshot = load_snapshot(&session_dir)?;
        
        // Update watched context files
        watched_context_files.clear();
        for context in &new_snapshot.code_contexts {
            if let Ok(uri) = url::Url::parse(&context.uri) {
                if let Ok(path) = uri.to_file_path() {
                    watched_context_files.insert(path);
                }
            }
        }
        
        snapshot.store(Arc::new(new_snapshot));
        eprintln!("[SNEK] Reloaded snapshot, version: {}", new_snapshot.version);
        Ok(())
    }
    
    fn update_contexts(
        snapshot: &Arc<ArcSwap<ContextSnapshot>>,
        modified_files: &HashSet<PathBuf>,
    ) {
        // Load current snapshot
        let current = snapshot.load();
        let mut new_snapshot = (**current).clone();
        
        // Update only the contexts that match modified files
        let mut updated_count = 0;
        for context in &mut new_snapshot.code_contexts {
            if let Ok(uri) = url::Url::parse(&context.uri) {
                if let Ok(path) = uri.to_file_path() {
                    if modified_files.contains(&path) {
                        if let Err(e) = update_context_from_file(context) {
                            eprintln!("[SNEK] Failed to update context {}: {}", context.uri, e);
                        } else {
                            updated_count += 1;
                        }
                    }
                }
            }
        }
        
        if updated_count > 0 {
            snapshot.store(Arc::new(new_snapshot));
            eprintln!("[SNEK] Updated {} context file(s)", updated_count);
        }
    }
}
```

**Key Design Decisions**:
- **Dual-mode watching**: Watches both session files (`session.json`, `chat.json`, `context.json`) and actual context files
- **Incremental context updates**: When a context file changes, only that file's content is re-read (not full reload)
- **Full reload**: When `context.json` changes, re-read everything and update watched files list
- **Debounce with 200ms timeout** to coalesce rapid file changes
- **Separate thread** for watch loop to avoid blocking LSP operations
- **HashSet tracking** of watched context files for efficient lookup
- **Graceful error handling**: Errors in context updates are logged but don't crash the watcher

**Watch Triggers**:
1. `active.json` changes → Full reload (session switch)
2. `session.json` changes → Full reload (metadata update)
3. `chat.json` changes → Full reload (chat history update)
4. `context.json` changes → Full reload (context list changed, rewatch files)
5. Context file changes → Incremental update (re-read only that file's lines)

**Performance Benefits**:
- Context file changes don't require re-reading session/chat files
- Only modified contexts are updated, not all contexts
- Watched file set is dynamically updated when `context.json` changes

**Dependencies to add**:
- `notify = "6"` for file system watching
- `arc-swap = "1"` for lock-free atomic snapshot swapping
- `url = "2"` for URI to file path conversion

### 4. Document Tracking

**Location**: `src/document_store.rs` (new file)

Track the currently active document's content:

```rust
use std::sync::RwLock;
use tower_lsp::lsp_types::{
    DidOpenTextDocumentParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    Position, Url,
};

#[derive(Default)]
pub struct DocumentStore {
    active_doc: RwLock<Option<DocumentContent>>,
}

struct DocumentContent {
    uri: Url,
    text: String,
    language_id: String,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn did_open(&self, params: DidOpenTextDocumentParams) {
        let content = DocumentContent {
            uri: params.text_document.uri,
            text: params.text_document.text,
            language_id: params.text_document.language_id,
        };
        *self.active_doc.write().unwrap() = Some(content);
    }
    
    pub fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut doc = self.active_doc.write().unwrap();
        if let Some(ref mut content) = *doc {
            if content.uri == params.text_document.uri {
                // Assuming full document sync
                if let Some(change) = params.content_changes.into_iter().next() {
                    content.text = change.text;
                }
            }
        }
    }
    
    pub fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut doc = self.active_doc.write().unwrap();
        if let Some(ref content) = *doc {
            if content.uri == params.text_document.uri {
                *doc = None;
            }
        }
    }
    
    /// Extract prefix and suffix around cursor position
    pub fn get_context(&self, uri: &Url, position: Position) -> Option<(String, String, String)> {
        let doc = self.active_doc.read().unwrap();
        let content = doc.as_ref()?;
        
        if &content.uri != uri {
            return None;
        }
        
        let lines: Vec<&str> = content.text.lines().collect();
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;
        
        if line_idx >= lines.len() {
            return None;
        }
        
        // Build prefix: all lines before + current line up to cursor
        let mut prefix = String::new();
        for (i, line) in lines.iter().enumerate() {
            if i < line_idx {
                prefix.push_str(line);
                prefix.push('\n');
            } else if i == line_idx {
                let line_prefix = line.chars().take(char_idx).collect::<String>();
                prefix.push_str(&line_prefix);
                break;
            }
        }
        
        // Build suffix: current line after cursor + all lines after
        let mut suffix = String::new();
        for (i, line) in lines.iter().enumerate() {
            if i == line_idx {
                let line_suffix = line.chars().skip(char_idx).collect::<String>();
                suffix.push_str(&line_suffix);
                suffix.push('\n');
            } else if i > line_idx {
                suffix.push_str(line);
                suffix.push('\n');
            }
        }
        
        Some((prefix, suffix, content.language_id.clone()))
    }
}
```

**Key Design Decisions**:
- Only tracks single active document (as per requirement 3b)
- Uses `RwLock` for thread-safe access (multiple reads, single write)
- `get_context()` splits document at cursor position into prefix/suffix
- Returns `Option` to handle cases where document isn't tracked
- Assumes `TextDocumentSyncKind::FULL` (entire document sent on each change)

### 5. Model Integration

**Location**: `src/model.rs` (new file)

Handle prompt assembly and OpenAI API calls:

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::snapshot::ContextSnapshot;

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: usize,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

pub struct ModelClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ModelClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }
    
    /// Assemble prompt from snapshot and buffer context
    fn build_messages(
        snapshot: &ContextSnapshot,
        prefix: &str,
        suffix: &str,
        language: &str,
    ) -> Vec<OpenAIMessage> {
        let mut messages = Vec::new();
        
        // Add system message
        messages.push(OpenAIMessage {
            role: "system".to_string(),
            content: format!(
                "You are an AI code completion assistant. Generate code that continues naturally from the cursor position. \
                 Language: {}. Rules: (1) Only output the code to insert, no explanations. \
                 (2) Match the existing code style. (3) Be concise and relevant.",
                language
            ),
        });
        
        // Add chat history from snapshot
        for msg in &snapshot.chat_messages {
            messages.push(OpenAIMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }
        
        // Build the final user message with buffer context and code contexts
        let mut context_msg = String::new();
        
        // Add code contexts if available
        if !snapshot.code_contexts.is_empty() {
            context_msg.push_str("Here are some other code snippets that might help you:\n\n");
            for (idx, ctx) in snapshot.code_contexts.iter().enumerate() {
                context_msg.push_str(&format!(
                    "Context {}:\n  URI: {}\n  Lines: {}-{}\n  Language: {}\n",
                    idx + 1, ctx.uri, ctx.start_line, ctx.end_line, ctx.language_id
                ));
                if let Some(ref desc) = ctx.description {
                    context_msg.push_str(&format!("  Description: {}\n", desc));
                }
                context_msg.push_str(&format!("  Code:\n```\n{}\n```\n\n", ctx.code));
            }
            context_msg.push_str("---\n\n");
        }
        
        // Add current buffer context
        context_msg.push_str(&format!(
            "Complete the code at the cursor position.\n\n\
             --- CODE BEFORE CURSOR ---\n{}\n\
             --- CODE AFTER CURSOR ---\n{}\n\n\
             Generate the code to insert at the cursor.",
            prefix, suffix
        ));
        
        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: context_msg,
        });
        
        messages
    }
    
    /// Call OpenAI-compatible API for completion
    pub async fn complete(
        &self,
        snapshot: &ContextSnapshot,
        prefix: &str,
        suffix: &str,
        language: &str,
    ) -> Result<String> {
        let messages = Self::build_messages(snapshot, prefix, suffix, language);
        
        let request = OpenAIRequest {
            model: "glm-4.6".to_string(), // Default model, configurable later
            messages,
            max_tokens: snapshot.limits.max_tokens,
            temperature: 0.0, // Lower temperature for more deterministic completions
            stream: false,
        };
        
        let url = format!("{}/chat/completions", self.base_url);
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to model API")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, body);
        }
        
        let api_response: OpenAIResponse = response.json().await
            .context("Failed to parse API response")?;
        
        let completion = api_response.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        
        Ok(completion)
    }
}
```

**Key Design Decisions**:
- OpenAI-compatible API format (works with OpenAI, Azure, local proxies, etc.)
- Base URL and API key configurable (will be provided by user later)
- Chat history from snapshot included in prompt for style/context awareness
- **Code contexts appended to final user message** (not scattered in chat history)
- Context format includes URI, line ranges, language, description, and code
- Prefix/suffix wrapped in clear delimiters for model understanding
- Low temperature (0.0) for consistent, predictable completions
- `max_tokens` from session limits
- Comprehensive error handling with context

**Dependencies to add**:
- `reqwest = { version = "0.12", features = ["json"] }` (already in Cargo.toml)

**TODO for later**:
- Make model name configurable
- Add retry logic for transient failures
- Support streaming responses for faster perceived latency
- Add timeout configuration

### 6. LSP Backend Updates

**Location**: `src/lsp/backend.rs` (modify existing)

Update the backend to integrate all components:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, async_trait};
use arc_swap::ArcSwap;

use crate::snapshot::ContextSnapshot;
use crate::document_store::DocumentStore;
use crate::model::ModelClient;

pub struct Backend {
    client: Client,
    snapshot: Arc<ArcSwap<ContextSnapshot>>,
    documents: DocumentStore,
    model: ModelClient,
}

impl Backend {
    pub fn new(
        client: Client,
        snapshot: Arc<ArcSwap<ContextSnapshot>>,
        model: ModelClient,
    ) -> Self {
        Self {
            client,
            snapshot,
            documents: DocumentStore::new(),
            model,
        }
    }
}

// Custom inline completion request
#[derive(Debug, Serialize, Deserialize)]
pub struct SnekInlineParams {
    pub uri: String,
    pub line: u32,
    pub character: u32,
    pub language_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnekInlineResponse {
    pub text: String,
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::FULL
            )),
            ..Default::default()
        };
        
        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "Snek LSP".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Snek LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.documents.did_open(params);
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.documents.did_change(params);
    }
    
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.did_close(params);
    }
}

// Handler for custom snek/inline method (to be registered in server.rs)
impl Backend {
    pub async fn handle_inline_completion(&self, params: Value) -> Result<Value> {
        let params: SnekInlineParams = serde_json::from_value(params)
            .map_err(|e| tower_lsp::jsonrpc::Error::invalid_params(e.to_string()))?;
        
        let uri = Url::parse(&params.uri)
            .map_err(|e| tower_lsp::jsonrpc::Error::invalid_params(e.to_string()))?;
        
        let position = Position {
            line: params.line,
            character: params.character,
        };
        
        // Get document context
        let (prefix, suffix, language) = self.documents
            .get_context(&uri, position)
            .unwrap_or_else(|| {
                (String::new(), String::new(), params.language_id.clone())
            });
        
        // Get current snapshot
        let snapshot = self.snapshot.load();
        
        // Call model
        let completion = self.model
            .complete(&snapshot, &prefix, &suffix, &language)
            .await
            .unwrap_or_else(|e| {
                // Log error and return empty completion
                let _ = self.client.log_message(
                    MessageType::ERROR,
                    format!("Model completion failed: {}", e)
                );
                String::new()
            });
        
        let response = SnekInlineResponse { text: completion };
        Ok(serde_json::to_value(response).unwrap())
    }
}
```

**Key Changes**:
- Add `snapshot`, `documents`, and `model` fields
- Implement `did_open`, `did_change`, `did_close` for document tracking
- Add `handle_inline_completion` method for custom request
- Set `TextDocumentSyncKind::FULL` for simpler document sync
- Remove old completion handler (replaced by custom method)

### 7. Server Setup

**Location**: `src/lsp/server.rs` (modify existing)

Update server initialization to wire all components:

```rust
use anyhow::Result;
use std::sync::Arc;
use tower_lsp::{LspService, Server};
use arc_swap::ArcSwap;

use super::backend::Backend;
use crate::session_io::find_workspace_root;
use crate::watcher::SessionWatcher;
use crate::snapshot::ContextSnapshot;
use crate::model::ModelClient;

pub async fn serve_stdio() -> Result<()> {
    // Find or create .snek/ directory
    let snek_root = find_workspace_root()?;
    
    // Initialize shared snapshot
    let snapshot = Arc::new(ArcSwap::from_pointee(ContextSnapshot::default()));
    
    // Start file watcher
    let _watcher = SessionWatcher::start(snek_root, snapshot.clone())?;
    
    // Initialize model client
    // TODO: Get these from config file or environment variables
    let model = ModelClient::new(
        std::env::var("SNEK_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
        std::env::var("SNEK_API_KEY")
            .unwrap_or_else(|_| "".to_string()),
    );
    
    // Build LSP service with custom method
    let (service, socket) = LspService::build(|client| {
        Backend::new(client, snapshot.clone(), model)
    })
    .custom_method("snek/inline", |backend: &Backend, params| {
        backend.handle_inline_completion(params)
    })
    .finish();
    
    // Serve over stdio
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
    
    Ok(())
}
```

**Key Changes**:
- Initialize `.snek/` directory structure on startup
- Create and start file watcher
- Initialize model client with environment variables
- Register custom `snek/inline` method
- Keep `_watcher` alive for duration of server

**Configuration**:
- `SNEK_API_BASE_URL`: OpenAI-compatible API base URL (user will provide)
- `SNEK_API_KEY`: API authentication key

### 8. Module Organization

**Location**: `src/lib.rs` (modify existing)

Update module exports:

```rust
pub mod lsp;
pub mod snapshot;
pub mod session_io;
pub mod watcher;
pub mod document_store;
pub mod model;
```

## Cargo.toml Updates

Add required dependencies:

```toml
[dependencies]
anyhow = "1.0"
tower-lsp = "0.20.0"
tokio = { version = "1.48.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json"] }
notify = "6"
arc-swap = "1"
chrono = "0.4"
url = "2"
uuid = { version = "1", features = ["v4"] }  # For generating session IDs

[dev-dependencies]
tempfile = "3"  # For testing with temporary directories
```

## Implementation Order

### Phase 1: Core Infrastructure (Foundation)
1. Create `src/snapshot.rs` with data structures
2. Create `src/session_io.rs` with file I/O functions
3. Test file reading/writing with unit tests
4. Verify `.snek/` initialization works

### Phase 2: File Watching (Reactivity)
5. Create `src/watcher.rs` with debounced watching
6. Implement context file watching and incremental updates
7. Test snapshot reloading on file changes
8. Test context file updates (modify watched file, verify snapshot updates)
9. Verify debouncing works correctly

### Phase 3: Document Tracking (Buffer Context)
10. Create `src/document_store.rs`
11. Update `backend.rs` with `did_open/change/close` handlers
12. Test prefix/suffix extraction at various cursor positions

### Phase 4: Model Integration (Intelligence)
13. Create `src/model.rs` with OpenAI client
14. Implement prompt assembly logic with code context support
15. Test with mock responses first, then real API

### Phase 5: LSP Integration (Wiring)
16. Update `backend.rs` with `handle_inline_completion`
17. Update `server.rs` with initialization logic
18. Update `lib.rs` with module exports
19. Update `Cargo.toml` with dependencies

### Phase 6: Testing & Refinement
20. End-to-end test: start LSP, call `snek/inline`, verify response
21. Test session switching (edit `active.json`, verify reload)
22. Test chat updates (edit `chat.json`, verify prompt changes)
23. Test context updates (edit `context.json`, verify watched files change)
24. Test context file modifications (edit watched file, verify incremental update)
25. Verify context appears in prompt correctly
26. Performance testing (measure completion latency)

## Testing Strategy

### Unit Tests

**File I/O** (`session_io.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_initialize_default_session() {
        let temp = TempDir::new().unwrap();
        let snek_root = temp.path().join(".snek");
        std::fs::create_dir(&snek_root).unwrap();
        
        initialize_default_session(&snek_root).unwrap();
        
        assert!(snek_root.join("active.json").exists());
        assert!(snek_root.join("sessions/default/session.json").exists());
        assert!(snek_root.join("sessions/default/chat.json").exists());
    }
    
    #[test]
    fn test_load_snapshot() {
        // Create test session files
        // Load snapshot
        // Verify fields match
    }
    
    #[test]
    fn test_update_context_from_file() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.rs");
        std::fs::write(&test_file, "line 0\nline 1\nline 2\nline 3\nline 4").unwrap();
        
        let mut context = CodeContext {
            uri: format!("file://{}", test_file.display()),
            start_line: 1,
            end_line: 3,
            language_id: "rust".to_string(),
            code: String::new(),
            description: None,
            last_modified: String::new(),
        };
        
        update_context_from_file(&mut context).unwrap();
        assert_eq!(context.code, "line 1\nline 2");
    }
}
```

**Document Store** (`document_store.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prefix_suffix_extraction() {
        let store = DocumentStore::new();
        // Open document with known content
        // Call get_context at various positions
        // Verify prefix/suffix correctness
    }
}
```

### Integration Tests

**Location**: `tests/integration_test.rs`

```rust
#[tokio::test]
async fn test_lsp_initialization() {
    // Start LSP server
    // Send initialize request
    // Verify capabilities response
}

#[tokio::test]
async fn test_inline_completion_flow() {
    // Setup .snek/ with test session
    // Start LSP
    // Send didOpen
    // Send snek/inline request
    // Verify response structure
}

#[tokio::test]
async fn test_session_reload() {
    // Start LSP with initial session
    // Modify chat.json
    // Wait for debounce
    // Send completion request
    // Verify new chat context is used
}
```

### Manual Testing Checklist

1. **Initialization**:
   - [ ] Start LSP in directory without `.snek/` → creates default structure
   - [ ] Start LSP in directory with `.snek/` → loads existing session
   - [ ] Verify log messages appear in LSP client

2. **Document Tracking**:
   - [ ] Open file → `didOpen` tracked
   - [ ] Edit file → `didChange` updates content
   - [ ] Close file → `didClose` clears tracking

3. **Inline Completion**:
   - [ ] Request completion → receives non-empty response
   - [ ] Verify response contains relevant code
   - [ ] Check latency is acceptable (<2s)

4. **Session Management**:
   - [ ] Edit `chat.json` → LSP reloads within 250ms
   - [ ] Switch `active.json` → LSP switches session
   - [ ] Verify version increments are detected

5. **Context Management**:
   - [ ] Add entry to `context.json` → LSP starts watching that file
   - [ ] Edit watched context file → LSP incrementally updates only that context
   - [ ] Remove entry from `context.json` → LSP stops watching that file
   - [ ] Verify context appears in completion prompts

6. **Error Handling**:
   - [ ] Invalid JSON in session files → graceful fallback
   - [ ] Missing API key → logs error, returns empty completion
   - [ ] Network timeout → logs error, doesn't crash

## Configuration

### Environment Variables

The LSP reads configuration from environment variables:

- `SNEK_API_BASE_URL`: Base URL for OpenAI-compatible API (default: `https://api.openai.com/v1`)
- `SNEK_API_KEY`: API authentication key (required for model calls)

### Future: Configuration File

Consider adding `.snek/config.json` for persistent configuration:

```json
{
  "model": {
    "base_url": "https://api.openai.com/v1",
    "model_name": "glm-4.6",
    "max_tokens": 1600,
    "temperature": 0.2
  },
  "lsp": {
    "debounce_ms": 200,
    "log_level": "info"
  }
}
```

## Performance Considerations

### Hot Path Optimization

The completion request hot path:
1. Parse request params (~0.1ms)
2. Read document from RAM cache (~0.1ms)
3. Load snapshot from Arc (~0.01ms)
4. Build prompt (~0.5ms)
5. Call model API (~500-2000ms) ← dominant factor
6. Return response (~0.1ms)

**Total**: ~500-2000ms (dominated by model inference)

### Memory Usage

- Snapshot: ~10KB per session (typical)
- Code contexts: ~5-50KB depending on number and size of contexts
- Document cache: ~100KB (single active document)
- Watcher overhead: ~1MB
- **Total**: ~2-10MB baseline (varies with context count)

### Optimization Opportunities

1. **Prompt caching**: Cache assembled prompt if snapshot hasn't changed
2. **Streaming responses**: Use SSE to return tokens as generated
3. **Speculative completion**: Pre-fetch completion on cursor idle
4. **Context truncation**: Limit prefix/suffix to last N lines

## Known Limitations & Future Work

### Current Limitations

1. **Single document tracking**: Only tracks active document, not all open files
2. **No streaming**: Completions wait for full model response
3. **No caching**: Every completion calls model API
4. **Environment-only config**: No persistent configuration file
5. **Context size limits**: Large contexts may exceed token limits
6. **No automatic context detection**: User must manually specify contexts in `context.json`

### Future Enhancements

1. **Automatic context detection**: Use LSP symbols/imports to auto-populate `context.json`
2. **Smart context selection**: Rank and select most relevant contexts based on imports/usage
3. **Context compression**: Summarize large contexts to fit within token limits
4. **Streaming completions**: Return tokens as they're generated
5. **Local model support**: Add llama.cpp or similar for offline use
6. **Diagnostics**: Add linting/error detection using chat context
7. **Code actions**: Suggest refactorings based on team style
8. **Telemetry**: Track completion acceptance rate
9. **Smart truncation**: Use tree-sitter to truncate at semantic boundaries
10. **Context caching**: Cache context embeddings for faster retrieval

## Security Considerations

1. **API Key Storage**: Currently from environment variable
   - TODO: Consider OS keychain integration
   - Never log or expose API key in responses

2. **File Permissions**: `.snek/` should be user-readable only
   - Contains potentially sensitive chat history
   - Consider adding permission checks on startup

3. **Model Input**: User code sent to external API
   - Document this clearly to users
   - Consider adding opt-out or local-only mode

## Debugging & Troubleshooting

### Enable Verbose Logging

Add logging to key operations:

```rust
// In session_io.rs
eprintln!("[SNEK] Loading snapshot from {:?}", session_dir);

// In watcher.rs
eprintln!("[SNEK] Reloaded snapshot, version: {}", new_snapshot.version);

// In model.rs
eprintln!("[SNEK] Calling model API: {} tokens", request.max_tokens);
```

### Common Issues

**Issue**: Completions return empty string
- Check: API key is set correctly
- Check: Network connectivity to API base URL
- Check: LSP logs for error messages

**Issue**: Snapshot not updating after file changes
- Check: File watcher is running (no panic in thread)
- Check: Debounce timing (wait 250ms after edit)
- Check: File permissions on `.snek/` directory

**Issue**: Context not updating when file changes
- Check: File is listed in `context.json` with correct URI
- Check: URI format is correct (file:// prefix)
- Check: Line ranges are valid (within file bounds)
- Check: File watcher has permission to watch the file

**Issue**: Wrong document content in completions
- Check: `didOpen` was called before completion request
- Check: Document URI matches between open and completion
- Check: `TextDocumentSyncKind::FULL` is set

## Success Criteria

The implementation is complete when:

1. ✅ LSP starts and initializes `.snek/` if missing (including `context.json`)
2. ✅ File watcher loads and reloads snapshots correctly
3. ✅ Context file watcher monitors all files in `context.json`
4. ✅ Context file changes trigger incremental updates (not full reload)
5. ✅ Document tracking captures prefix/suffix accurately
6. ✅ `snek/inline` returns model-generated completions
7. ✅ Chat history influences completion style
8. ✅ Code contexts appear in final prompt after chat history
9. ✅ Session switching works within 300ms
10. ✅ No disk I/O on completion hot path (except context file updates)
11. ✅ Errors are logged and don't crash the server

## Timeline Estimate

- **Phase 1** (Core Infrastructure): 3-4 hours
- **Phase 2** (File Watching + Context Monitoring): 5-7 hours
- **Phase 3** (Documents): 2-3 hours
- **Phase 4** (Model + Context Integration): 4-5 hours
- **Phase 5** (LSP Integration): 2-3 hours
- **Phase 6** (Testing & Refinement): 4-6 hours

**Total**: 20-28 hours for full implementation (including context management)

## Next Steps

After LSP implementation is complete:

1. **CLI Implementation**: Build `snek-cli` for session/chat management
2. **VS Code Extension**: Build TypeScript extension to call `snek/inline`
3. **Documentation**: Write user guide and API documentation
4. **Packaging**: Create installation scripts and releases

---

## Appendix: API Contract

### Custom LSP Method: `snek/inline`

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "snek/inline",
  "params": {
    "uri": "file:///path/to/file.rs",
    "line": 42,
    "character": 15,
    "language_id": "rust"
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "text": "fn example() {\n    println!(\"Hello\");\n}"
  }
}
```

**Error Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Model API call failed: timeout"
  }
}
```

---

## Appendix B: Context Management Workflow

### Adding Context via CLI/Extension

Users (or the CLI/Extension) can add code contexts by editing `context.json`:

```bash
# Example: Add a context manually
cat >> .snek/sessions/<session-id>/context.json <<EOF
{
  "schema": 1,
  "contexts": [
    {
      "uri": "file:///path/to/project/src/models.rs",
      "start_line": 0,
      "end_line": 50,
      "language_id": "rust",
      "code": "",
      "description": "User model definitions",
      "last_modified": "2025-11-02T10:00:00Z"
    }
  ]
}
EOF
```

The LSP will:
1. Detect the change to `context.json` (via file watcher)
2. Reload the full snapshot
3. Start watching `file:///path/to/project/src/models.rs`
4. Extract lines 0-50 from the file and populate the `code` field
5. Use this context in all subsequent completions

### Context Update Flow

```
User edits models.rs
    ↓
File watcher detects change
    ↓
Watcher identifies it as a context file (in watched_context_files HashSet)
    ↓
Debounce for 200ms
    ↓
Call update_contexts() with modified file path
    ↓
Find matching context(s) by URI
    ↓
Call update_context_from_file() for each match
    ↓
Extract lines [start_line..end_line] from file
    ↓
Update context.code and context.last_modified
    ↓
Store new snapshot with updated context
    ↓
Next completion request uses fresh context
```

### Example Prompt with Context

Given:
- Chat history: 2 messages about coding style
- Code contexts: 1 context from models.rs
- Current file: main.rs at line 50

The final prompt to the model will be:

```
System: You are an AI code completion assistant...

User: We use snake_case for functions
Assistant: Noted. I'll follow that convention.

User: Here are some other code snippets that might help you:

Context 1:
  URI: file:///path/to/project/src/models.rs
  Lines: 10-30
  Language: rust
  Description: User model definitions
  Code:
```
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: String, name: String, email: String) -> Self {
        Self { id, name, email }
    }
}
```

---

Complete the code at the cursor position.

--- CODE BEFORE CURSOR ---
fn main() {
    let user = 

--- CODE AFTER CURSOR ---
    println!("{}", user.name);
}

Generate the code to insert at the cursor.
```

The model can now see the `User` struct definition and generate appropriate code like:
```rust
User::new("123".to_string(), "Alice".to_string(), "alice@example.com".to_string());
```

---

*This plan provides a complete roadmap for implementing the Snek LSP with context management. Follow the phases in order, test thoroughly at each stage, and refer back to this document for architectural decisions and design rationale.*

