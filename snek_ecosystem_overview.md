# 0) Files on disk (the whole contract)

```
.snek/
  active.json
  sessions/
    <session-id>/
      session.json
      chat.json
```

## `active.json`

* Single source of truth for which session is active.

```json
{
  "schema": 1,
  "id": "b4ubg485",
  "path": "sessions/b4ubg485"
}
```

## `session.json`

* Tiny metadata + versioning (so LSP can rebuild cache safely).

```json
{
  "schema": 1,
  "id": "b4ubg485",
  "name": "default",
  "version": 42,                  // bump AFTER chat.json saves
  "limits": { "max_tokens": 1600 },
  "updated_at": "2025-10-31T17:22:11Z"
}
```

## `chat.json`

* Standard chat history array (easy to inject into the model directly).

```json
{
  "schema": 1,
  "messages": [
    { "role": "system", "content": "You are Snek; write concise code." },
    { "role": "user",   "content": "We use Actix; prefer snake_case." },
    { "role": "assistant", "content": "Noted. Avoid async_trait." }
  ]
}
```

> **Atomic write rule for writers (CLI/extension):** write to `*.tmp` → rename → then bump `session.json.version` (also atomic).
> This guarantees the LSP never reads partial JSON.

---

# 1) Division of responsibilities

## VS Code Extension (TypeScript)

* Starts the **Rust LSP** (stdio).
* Provides **inline completions** UI by calling a **custom LSP method** (`snek/inline`).
* Adds a few commands that **edit `chat.json`** or **switch sessions** by writing `active.json` (atomic rename).
  No direct “notify LSP” needed; the LSP watches the files.

## Snek LSP (Rust, `tower-lsp`)

* Keeps an **in-RAM snapshot** of `{ session_id, version, chat_messages, limits }`.
* **Watches** `.snek/active.json` and the resolved session folder (`sessions/<id>/`) with a debounce.
* On `snek/inline`:

  * Reads current buffer text (prefix/suffix around cursor) from its open-doc cache.
  * Pulls `chat_messages` and `limits` from **RAM snapshot**.
  * Calls the model and returns the suggestion (ghost text).
* Later: add diagnostics/linting using the same snapshot + buffer text.

## Snek CLI (Rust, `clap`)

* Session management and chat editing as **file operations** only:

  * `snek session new|use`
  * `snek chat add|set|clear`
* Every write is atomic; then it **bumps `session.json.version`** (atomic).

---

# 2) How they connect (no IPC, filesystem only)

1. **Extension** launches the LSP (stdio).
2. **LSP** on startup loads `.snek/active.json` → resolves `sessions/<id>/` → reads `session.json` + `chat.json` → builds RAM snapshot.
3. **LSP** starts file watchers on:

   * `.snek/active.json`
   * `.snek/sessions/<id>/session.json`
   * `.snek/sessions/<id>/chat.json`
4. **CLI or Extension** edits `chat.json` or switches `active.json` (atomic) → **bumps version in `session.json`**.
5. **LSP watcher** sees change → debounces (e.g., 150–250 ms) → **reloads** into RAM snapshot.
6. User types → Extension asks `snek/inline{uri,line,char,language_id}` → LSP replies using **prefix/suffix + snapshot.chat**.

---

# 3) Implementation details

## 3.1 LSP (Rust) – crates

```toml
# snek-lsp/Cargo.toml
[dependencies]
anyhow = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tower-lsp = "0.20"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
notify = "6"
arc-swap = "1"
reqwest = { version = "0.12", features = ["json", "stream"] } # if calling hosted model
```

### Snapshot types

```rust
// snapshot.rs
#[derive(Clone)]
pub struct Limits { pub max_tokens: usize }

#[derive(Clone)]
pub struct ContextSnapshot {
    pub session_id: String,
    pub version: u64,
    pub limits: Limits,
    pub chat_messages: Vec<ChatMsg>,
}
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ChatMsg { pub role: String, pub content: String }
```

### Watch + load

```rust
// context.rs
use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::PathBuf, sync::mpsc::channel};
use crate::snapshot::*;

pub fn start_watch(root: PathBuf, shared: arc_swap::ArcSwap<ContextSnapshot>) -> Result<RecommendedWatcher> {
    let (tx, rx) = channel();
    let mut w = notify::recommended_watcher(move |e| { let _ = tx.send(e); })?;
    // watch active.json and current session files
    w.watch(&root.join("active.json"), RecursiveMode::NonRecursive)?;
    // initial: read active -> session dir -> load snapshot
    let mut session_dir = resolve_active(&root)?; // read active.json
    shared.store(Arc::new(load_snapshot(&session_dir)?));

    // also watch those two files:
    let watch_session_files = |w: &mut RecommendedWatcher, dir: &PathBuf| -> Result<()> {
        w.watch(&dir.join("session.json"), RecursiveMode::NonRecursive)?;
        w.watch(&dir.join("chat.json"), RecursiveMode::NonRecursive)?;
        Ok(())
    };
    watch_session_files(&mut w, &session_dir)?;

    // debounce loop
    std::thread::spawn(move || {
        use std::time::{Duration, Instant}; let mut last = Instant::now();
        loop {
            if rx.recv().is_err() { break; }
            if last.elapsed() < Duration::from_millis(200) { continue; }
            last = Instant::now();

            // if active.json changed, swap session_dir + rewatch
            if let Ok(new_dir) = resolve_active(&root) {
                if new_dir != session_dir {
                    session_dir = new_dir;
                    let _ = w.unwatch(session_dir.join("session.json"));
                    let _ = w.unwatch(session_dir.join("chat.json"));
                    let _ = watch_session_files(&mut w, &session_dir);
                }
            }
            if let Ok(snap) = load_snapshot(&session_dir) {
                shared.store(Arc::new(snap));
            }
        }
    });
    Ok(w)
}

fn load_snapshot(dir: &PathBuf) -> Result<ContextSnapshot> {
    // read session.json (get id/version/limits)
    // read chat.json (messages)
    // if chat.json missing, default to []
    // return ContextSnapshot { ... }
    todo!()
}

fn resolve_active(root: &PathBuf) -> Result<PathBuf> {
    // read .snek/active.json → {"path": "sessions/<id>"}
    // return root.join(path)
    todo!()
}
```

### LSP server + inline method

```rust
// main.rs
use anyhow::Result;
use tower_lsp::{LspService, Server, Client, LanguageServer};
use tower_lsp::lsp_types::*;
use std::sync::Arc;
mod context; mod snapshot; mod model; mod open_docs;

struct Backend {
  client: Client,
  snap: Arc<arc_swap::ArcSwap<snapshot::ContextSnapshot>>,
  docs: open_docs::DocStore, // cache of open file texts
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, _p: InitializeParams) -> Result<InitializeResult> {
    Ok(InitializeResult {
      capabilities: ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions::default()),
        ..Default::default()
      }, server_info: None
    })
  }
  async fn did_open(&self, p: DidOpenTextDocumentParams) { self.docs.open(p); }
  async fn did_change(&self, p: DidChangeTextDocumentParams) { self.docs.change(p); }
}

#[tokio::main]
async fn main() -> Result<()> {
  let root = crate::util::find_workspace_root()?;   // locate .snek/
  let shared = Arc::new(arc_swap::ArcSwap::from_pointee(default_snapshot()));
  let _watcher = context::start_watch(root, shared.clone())?;

  let (service, socket) = LspService::build(|client| Backend {
      client, snap: shared.clone(), docs: open_docs::DocStore::default()
    })
    .custom_method("snek/inline", |srv: &Backend, req: serde_json::Value| async move {
      // req: {uri,line,character,language_id}
      let (prefix, suffix, lang) = srv.docs.slice(&req);
      let snap = srv.snap.load();
      let prompt = model::assemble(&snap, &prefix, &suffix, &lang);
      let text = model::infer(&prompt).await.unwrap_or_default();
      Ok(serde_json::json!({ "text": text }))
    }).finish();

  Server::new(tokio::io::stdin(), tokio::io::stdout(), socket).serve(service).await;
  Ok(())
}
```

### Prompt assembly (chat + buffer)

```rust
// model.rs
use crate::snapshot::ContextSnapshot;

pub fn assemble(snap: &ContextSnapshot, prefix: &str, suffix: &str, lang: &str) -> String {
  let chat = snap.chat_messages.iter()
    .map(|m| format!("{}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n");
  format!(
"System: You are an inline code completion engine for {lang}.
Rules: continue exactly at cursor; match style; no extra commentary.
Chat:
{chat}

--- PREFIX ---
{prefix}
--- SUFFIX ---
{suffix}
"
  )
}

pub async fn infer(prompt: &str) -> anyhow::Result<String> {
  // Call your model here (hosted or local). For MVP, return a stub.
  Ok("// TODO".into())
}
```

---

## 3.2 CLI (Rust) — commands as file writers

### Crates

```toml
# snek-cli/Cargo.toml
[dependencies]
clap = { version = "4", features = ["derive"] }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
```

### Commands

```rust
// main.rs
use clap::{Parser, Subcommand}; use anyhow::Result;

#[derive(Parser)]
struct Cli { #[command(subcommand)] cmd: Cmd }

#[derive(Subcommand)]
enum Cmd {
  SessionNew { name: String },            // creates sessions/<id>/, writes active.json
  SessionUse { name_or_id: String },      // updates active.json (atomic)
  ChatAdd    { text: String },            // append new assistant/user/system entry
  ChatSet    { path: Option<String> },    // replace chat.json from file/stdin
  ChatClear,                               // reset to empty []
}

fn main() -> Result<()> {
  let cli = Cli::parse();
  match cli.cmd {
    Cmd::SessionNew { name }         => session_new(name)?,
    Cmd::SessionUse { name_or_id }   => session_use(name_or_id)?,
    Cmd::ChatAdd { text }            => chat_add("user", text)?,     // or flag for role
    Cmd::ChatSet { path }            => chat_set(path)?,
    Cmd::ChatClear                   => chat_clear()?,
  }
  bump_version()?;                   // always bump after writes
  Ok(())
}
```

**Writers pattern (all of them):**

* read `.snek/active.json` to locate session dir
* update `chat.json.tmp` or `active.json.tmp`
* `rename(tmp, final)` (atomic)
* then read `session.json`, `version += 1`, write `session.json.tmp`, rename

---

# 4) Extension commands (TypeScript)

* **Add chat message** (user prompt to set style/team norms):

  * Read `.snek/active.json` → open `chat.json` → add `{"role":"user","content":"…"}` → save atomically → bump `session.json.version`.
* **Switch session**:

  * Write new `.snek/active.json.tmp` with `{id,path}` → rename → done.
* **Inline completion**:

  * `client.sendRequest('snek/inline', { uri, line, character, language_id })`.

*(You can share the atomic writer code between CLI and extension via a tiny TS helper or just reimplement it—both are short.)*

---

# 5) Latency & robustness

* **No per-completion disk I/O**: the LSP uses the RAM snapshot and the open buffer cache.
* **Debounce file watcher** (150–250 ms) to coalesce bursts.
* **Validate JSON** before rename to avoid corrupting the active session.
* **Size limits**:

  * Keep only the **last N chat messages** (e.g., last 10) in the snapshot if you want to control prompt length.
  * Use `limits.max_tokens` from `session.json` to cap model output.

---

# 6) Test matrix

* New session → `active.json` points to it → LSP snapshot loads.
* Add chat lines rapidly → single snapshot reload after debounce; `version` increments.
* Switch sessions → LSP reloads with new chat in <300 ms.
* Typing → completions return using chat-guided style.

---

## TL;DR

* **Session folder contains only `session.json` and `chat.json`.**
* **Active session is selected via `active.json` (no symlink).**
* **CLI/Extension only write these files atomically and bump `version`.**
* **LSP watches them, keeps a RAM snapshot, and serves completions with `chat.json` history + buffer prefix/suffix.**

This gives you exactly what you want: minimal files, minimal moving parts, and a fast hot path.
