use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::snapshot::{CodeContext, ContextSnapshot, Limits};

#[derive(Deserialize)]
#[allow(dead_code)]
struct ActiveJson {
    schema: u32,
    id: String,
    path: String,
}

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

#[derive(Deserialize)]
#[allow(dead_code)]
struct CodeSnippetsJson {
    schema: u32,
    snippets: Vec<CodeContext>,
}

pub fn find_workspace_root(workspace_dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(workspace) = workspace_dir {
        let snek_dir = workspace.join(".snek");

        if snek_dir.exists() && snek_dir.is_dir() {
            return Ok(snek_dir);
        }

        std::fs::create_dir_all(&snek_dir)?;
        initialize_default_session(&snek_dir)?;
        return Ok(snek_dir);
    }

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
                let snek_dir = current.join(".snek");
                std::fs::create_dir_all(&snek_dir)?;
                initialize_default_session(&snek_dir)?;
                return Ok(snek_dir);
            }
        }
    }
}

fn initialize_default_session(snek_root: &Path) -> Result<()> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_dir = snek_root.join("sessions").join(&session_id);
    std::fs::create_dir_all(&session_dir)?;
    std::fs::create_dir_all(session_dir.join("context"))?;
    std::fs::create_dir_all(snek_root.join("scripts"))?;
    std::fs::create_dir_all(snek_root.join("commands"))?;

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

    let snippets = serde_json::json!({
        "schema": 1,
        "snippets": []
    });
    std::fs::write(
        session_dir.join("code_snippets.json"),
        serde_json::to_string_pretty(&snippets)?,
    )?;

    let active = serde_json::json!({
        "schema": 1,
        "id": session_id,
        "path": format!("sessions/{}", session_id)
    });
    std::fs::write(
        snek_root.join("active.json"),
        serde_json::to_string_pretty(&active)?,
    )?;

    // Scripts
    write_script_file(snek_root, "scripts/new-session.sh", include_str!("../templates/scripts/new-session.sh"))?;
    write_script_file(snek_root, "scripts/switch-session.sh", include_str!("../templates/scripts/switch-session.sh"))?;
    write_script_file(snek_root, "scripts/add-code-snippet.sh", include_str!("../templates/scripts/add-code-snippet.sh"))?;
    write_script_file(snek_root, "scripts/add-context-file.sh", include_str!("../templates/scripts/add-context-file.sh"))?;
    write_script_file(snek_root, "scripts/clear-context.sh", include_str!("../templates/scripts/clear-context.sh"))?;
    write_script_file(snek_root, "scripts/clone-session.sh", include_str!("../templates/scripts/clone-session.sh"))?;
    write_script_file(snek_root, "scripts/get-active-session.sh", include_str!("../templates/scripts/get-active-session.sh"))?;
    write_script_file(snek_root, "scripts/get-file-outline.sh", include_str!("../templates/scripts/get-file-outline.sh"))?;
    write_script_file(snek_root, "scripts/get-git-diff.sh", include_str!("../templates/scripts/get-git-diff.sh"))?;
    write_script_file(snek_root, "scripts/get-git-status.sh", include_str!("../templates/scripts/get-git-status.sh"))?;
    write_script_file(snek_root, "scripts/get-imports.sh", include_str!("../templates/scripts/get-imports.sh"))?;
    write_script_file(snek_root, "scripts/get-recent-commits.sh", include_str!("../templates/scripts/get-recent-commits.sh"))?;
    write_script_file(snek_root, "scripts/list-context.sh", include_str!("../templates/scripts/list-context.sh"))?;
    write_script_file(snek_root, "scripts/list-sessions.sh", include_str!("../templates/scripts/list-sessions.sh"))?;
    write_script_file(snek_root, "scripts/remove-code-snippet.sh", include_str!("../templates/scripts/remove-code-snippet.sh"))?;
    write_script_file(snek_root, "scripts/remove-context-file.sh", include_str!("../templates/scripts/remove-context-file.sh"))?;
    write_script_file(snek_root, "scripts/snek-parse.sh", include_str!("../templates/scripts/snek-parse.sh"))?;

    // Commands
    write_script_file(snek_root, "commands/snek.share.md", include_str!("../templates/commands/snek.share.md"))?;
    write_script_file(snek_root, "commands/snek.blame.md", include_str!("../templates/commands/snek.blame.md"))?;
    write_script_file(snek_root, "commands/snek.callers.md", include_str!("../templates/commands/snek.callers.md"))?;
    write_script_file(snek_root, "commands/snek.commit.draft.md", include_str!("../templates/commands/snek.commit.draft.md"))?;
    write_script_file(snek_root, "commands/snek.commits.md", include_str!("../templates/commands/snek.commits.md"))?;
    write_script_file(snek_root, "commands/snek.complexity.md", include_str!("../templates/commands/snek.complexity.md"))?;
    write_script_file(snek_root, "commands/snek.context.add.md", include_str!("../templates/commands/snek.context.add.md"))?;
    write_script_file(snek_root, "commands/snek.context.clear.md", include_str!("../templates/commands/snek.context.clear.md"))?;
    write_script_file(snek_root, "commands/snek.context.read.md", include_str!("../templates/commands/snek.context.read.md"))?;
    write_script_file(snek_root, "commands/snek.context.remove.md", include_str!("../templates/commands/snek.context.remove.md"))?;
    write_script_file(snek_root, "commands/snek.context.show.md", include_str!("../templates/commands/snek.context.show.md"))?;
    write_script_file(snek_root, "commands/snek.deps.md", include_str!("../templates/commands/snek.deps.md"))?;
    write_script_file(snek_root, "commands/snek.diff.md", include_str!("../templates/commands/snek.diff.md"))?;
    write_script_file(snek_root, "commands/snek.doc.file.md", include_str!("../templates/commands/snek.doc.file.md"))?;
    write_script_file(snek_root, "commands/snek.doc.function.md", include_str!("../templates/commands/snek.doc.function.md"))?;
    write_script_file(snek_root, "commands/snek.explain.md", include_str!("../templates/commands/snek.explain.md"))?;
    write_script_file(snek_root, "commands/snek.fill.md", include_str!("../templates/commands/snek.fill.md"))?;
    write_script_file(snek_root, "commands/snek.outline.md", include_str!("../templates/commands/snek.outline.md"))?;
    write_script_file(snek_root, "commands/snek.refactor.extract.md", include_str!("../templates/commands/snek.refactor.extract.md"))?;
    write_script_file(snek_root, "commands/snek.refactor.rename.md", include_str!("../templates/commands/snek.refactor.rename.md"))?;
    write_script_file(snek_root, "commands/snek.refs.md", include_str!("../templates/commands/snek.refs.md"))?;
    write_script_file(snek_root, "commands/snek.session.clone.md", include_str!("../templates/commands/snek.session.clone.md"))?;
    write_script_file(snek_root, "commands/snek.session.info.md", include_str!("../templates/commands/snek.session.info.md"))?;
    write_script_file(snek_root, "commands/snek.session.list.md", include_str!("../templates/commands/snek.session.list.md"))?;
    write_script_file(snek_root, "commands/snek.session.new.md", include_str!("../templates/commands/snek.session.new.md"))?;
    write_script_file(snek_root, "commands/snek.session.switch.md", include_str!("../templates/commands/snek.session.switch.md"))?;
    write_script_file(snek_root, "commands/snek.status.md", include_str!("../templates/commands/snek.status.md"))?;
    write_script_file(snek_root, "commands/snek.test.generate.md", include_str!("../templates/commands/snek.test.generate.md"))?;
    write_script_file(snek_root, "commands/snek.todo.md", include_str!("../templates/commands/snek.todo.md"))?;

    Ok(())
}

fn write_script_file(snek_root: &Path, relative_path: &str, content: &str) -> Result<()> {
    let file_path = snek_root.join(relative_path);
    if !file_path.exists() {
        std::fs::write(&file_path, content)?;

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

pub fn resolve_active_session(snek_root: &Path) -> Result<PathBuf> {
    let active_path = snek_root.join("active.json");
    let content = std::fs::read_to_string(&active_path).context("Failed to read active.json")?;
    let active: ActiveJson =
        serde_json::from_str(&content).context("Failed to parse active.json")?;

    Ok(snek_root.join(&active.path))
}

pub fn load_snapshot(session_dir: &Path) -> Result<ContextSnapshot> {
    let session_path = session_dir.join("session.json");
    let session_content =
        std::fs::read_to_string(&session_path).context("Failed to read session.json")?;
    let session: SessionJson =
        serde_json::from_str(&session_content).context("Failed to parse session.json")?;

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

    let mut file_cache = std::collections::HashMap::new();
    for snippet in &code_snippets {
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
