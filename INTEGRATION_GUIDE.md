# Snek LSP Integration Guide

This guide explains how to integrate the Snek LSP binary with any IDE or editor extension.

## Core Concept

The Snek LSP is a **standalone binary** that communicates via stdin/stdout (LSP protocol). It can be integrated into any editor that supports LSP.

---

## Required Arguments

The LSP binary accepts the following command-line arguments:

### `--workspace-dir <path>` (Required)

Specifies the workspace/project root directory where the `.snek/` folder should be created.

**Examples:**
```bash
# Standard format
snek --workspace-dir /path/to/project

# Alternative format (with =)
snek --workspace-dir=/path/to/project

# Short format
snek --workspace /path/to/project
```

**Why this is required:**
- The LSP creates a `.snek/` folder in the workspace to store:
  - `settings.json` - API key and configuration
  - `sessions/{id}/` - Session data, context files, code snippets
  - `scripts/` - Utility scripts

---

## Configuration

### 1. API Key (Required for AI completions)

The LSP reads the API key from **two sources** (in priority order):

1. **`.snek/settings.json`** in the workspace (recommended)
   ```json
   {
     "schema": 1,
     "api_key": "your-api-key-here"
   }
   ```

2. **Environment variable** `SNEK_API_KEY` (fallback)
   ```bash
   export SNEK_API_KEY="your-api-key-here"
   snek --workspace-dir /path/to/project
   ```

**Behavior without API key:**
- LSP starts successfully
- Shows a helpful error message in logs
- Waits for user to configure API key
- Automatically reloads when `settings.json` changes

### 2. API URL and Model (Hardcoded)

Currently hardcoded in the LSP:
- API URL: `https://openai-proxy-aifp.onrender.com/v1/chat/completions`
- Model: `glm-4.6`

---

## Integration Examples

### VS Code Extension (TypeScript)

```typescript
import { LanguageClient, ServerOptions, TransportKind } from 'vscode-languageclient/node';

const serverOptions: ServerOptions = {
  command: '/path/to/snek',
  args: ['--workspace-dir', workspaceFolder.uri.fsPath],
  transport: TransportKind.stdio,
  options: {
    env: process.env // Optional: pass SNEK_API_KEY if configured
  }
};

const client = new LanguageClient(
  'snekLsp',
  'Snek Language Server',
  serverOptions,
  clientOptions
);

await client.start();
```

### JetBrains Plugin (Kotlin)

```kotlin
import com.intellij.openapi.project.Project
import com.intellij.platform.lsp.api.LspServerManager

val lspServerDescriptor = object : ProjectWideLspServerDescriptor(project, "Snek LSP") {
    override fun createCommandLine(): GeneralCommandLine {
        return GeneralCommandLine(
            "/path/to/snek",
            "--workspace-dir",
            project.basePath ?: ""
        )
    }
}

LspServerManager.getInstance(project).startServer(lspServerDescriptor)
```

### Neovim (Lua)

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

configs.snek = {
  default_config = {
    cmd = { 'snek', '--workspace-dir', vim.fn.getcwd() },
    filetypes = { 'rust', 'python', 'javascript', 'typescript', 'java', 'go', 'c', 'cpp' },
    root_dir = function(fname)
      return vim.fn.getcwd()
    end,
  },
}

lspconfig.snek.setup{}
```

### Emacs (Elisp)

```elisp
(require 'lsp-mode)

(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection
                   (lambda ()
                     (list "snek" "--workspace-dir" (projectile-project-root))))
  :major-modes '(rust-mode python-mode js-mode typescript-mode)
  :server-id 'snek))

(add-hook 'rust-mode-hook #'lsp)
(add-hook 'python-mode-hook #'lsp)
```

---

## LSP Custom Methods

### `snek/inline` - Get inline completion

**Request:**
```json
{
  "text_document": {
    "uri": "file:///path/to/file.rs"
  },
  "position": {
    "line": 10,
    "character": 5
  }
}
```

**Response:**
```json
{
  "completion": "let x = 42;"
}
```

---

## File Structure

When integrated, the LSP creates this structure in the workspace:

```
project/
├── .snek/
│   ├── active.json              # Current active session
│   ├── settings.json            # API key and config
│   ├── scripts/                 # Utility scripts
│   │   ├── new-session.sh
│   │   └── switch-session.sh
│   └── sessions/
│       └── {uuid}/
│           ├── session.json     # Session metadata
│           ├── code_snippets.json  # Code context references
│           └── context/         # Markdown context files
│               ├── architecture.md
│               └── conventions.md
└── your-project-files...
```

---

## Session Management

### Active Session

The LSP loads the session specified in `.snek/active.json`:
```json
{
  "schema": 1,
  "id": "aaf82595-38b4-4aef-a2c0-f7b4c2ffabae",
  "path": "sessions/aaf82595-38b4-4aef-a2c0-f7b4c2ffabae"
}
```

### Creating New Sessions

Use the provided script:
```bash
.snek/scripts/new-session.sh "my-session-name"
```

### Switching Sessions

Use the provided script:
```bash
.snek/scripts/switch-session.sh aaf82595  # First 8 chars of UUID
```

**Note:** After switching sessions, restart the LSP (reload your IDE).

---

## File Watching

The LSP automatically watches these files:

1. **`.snek/settings.json`** - Reloads API key when changed
2. **`.snek/active.json`** - Detects session switches
3. **`.snek/sessions/{id}/code_snippets.json`** - Updates code context
4. **`.snek/sessions/{id}/context/*.md`** - Updates markdown context
5. **Referenced code files** - Updates when source files change

**Important:** Changes to `settings.json` or `active.json` require an LSP restart.

---

## Logs

The LSP outputs logs to stderr:

```bash
[SNEK] Starting Snek Language Server...
[SNEK] Workspace directory provided: /path/to/project
[SNEK] Initializing workspace...
[SNEK] Workspace root: "/path/to/project/.snek"
[SNEK] Active session: "/path/to/project/.snek/sessions/{uuid}"
[SNEK] Loaded session: {uuid} (version 0)
[SNEK] Starting file watcher...
[SNEK] API key loaded successfully
[SNEK] Server ready, listening on stdio...
```

### Error: No API Key

```
╔════════════════════════════════════════════════════════════════════╗
║                         ⚠️  API KEY MISSING  ⚠️                      ║
╠════════════════════════════════════════════════════════════════════╣
║ Snek LSP server started but API key is not configured.            ║
║                                                                    ║
║ To enable AI completions, add your API key to:                    ║
║   .snek/settings.json                                              ║
║                                                                    ║
║ Example settings.json:                                             ║
║   {                                                                ║
║     "schema": 1,                                                   ║
║     "api_key": "your-api-key-here"                                ║
║   }                                                                ║
╚════════════════════════════════════════════════════════════════════╝
```

---

## Supported Languages

The LSP provides completions for:
- Rust (`.rs`)
- Python (`.py`)
- JavaScript (`.js`)
- TypeScript (`.ts`)
- Java (`.java`)
- Go (`.go`)
- C (`.c`)
- C++ (`.cpp`, `.cc`, `.cxx`)

---

## Binary Distribution

The binary is platform-specific:
- **macOS (ARM64)**: `snek` (Mach-O 64-bit executable arm64)
- **macOS (x86_64)**: Compile with `cargo build --target x86_64-apple-darwin`
- **Linux**: Compile with `cargo build --target x86_64-unknown-linux-gnu`
- **Windows**: Compile with `cargo build --target x86_64-pc-windows-msvc`

### Building for Different Platforms

```bash
# macOS ARM64 (M1/M2/M3)
cargo build --release

# macOS Intel
cargo build --release --target x86_64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

---

## Troubleshooting

### LSP doesn't start

1. Check the binary is executable: `chmod +x /path/to/snek`
2. Verify workspace directory exists: `ls -la /path/to/workspace`
3. Check LSP logs (stderr) for error messages

### `.snek` folder created in wrong location

- Ensure `--workspace-dir` argument is passed correctly
- Verify the path is absolute, not relative

### No completions appearing

1. Check API key is configured in `.snek/settings.json`
2. Look for errors in LSP logs
3. Verify the file type is supported
4. Check network connectivity to API endpoint

### Session not found error

```bash
# Check active session
cat .snek/active.json

# List available sessions
ls .snek/sessions/

# Fix by pointing to an existing session or creating a new one
.snek/scripts/new-session.sh
```

---

## Advanced: Custom Context

Add custom context for better completions:

1. **Markdown files** in `.snek/sessions/{id}/context/`:
   ```bash
   echo "# Project Conventions

   - Use snake_case for variables
   - All functions must have docstrings" > .snek/sessions/{id}/context/conventions.md
   ```

2. **Code snippets** in `.snek/sessions/{id}/code_snippets.json`:
   ```json
   {
     "schema": 1,
     "snippets": [
       {
         "uri": "file:///path/to/project/src/utils.rs",
         "start_line": 0,
         "end_line": 50,
         "language_id": "rust",
         "description": "Utility functions",
         "last_modified": "2025-11-12T10:00:00Z"
       }
     ]
   }
   ```

---

## Summary

To integrate Snek LSP with any editor:

1. **Launch the binary** with `--workspace-dir` argument
2. **Communicate via stdio** using LSP protocol
3. **Configure API key** in `.snek/settings.json`
4. **Call `snek/inline`** for completions

That's it! The LSP handles everything else automatically.
