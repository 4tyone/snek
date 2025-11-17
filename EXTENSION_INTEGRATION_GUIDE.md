# Snek LSP - Extension Integration Guide

**Version**: 0.1.0  
**Last Updated**: November 3, 2025  
**Target Audience**: IDE Extension Developers

---

## Overview

This guide provides all the technical details needed to integrate the Snek Language Server Protocol (LSP) into any IDE or editor. It covers:

- LSP server lifecycle and communication
- Standard and custom LSP methods
- File system contracts for configuration
- Session management
- Error handling and debugging

**What You'll Build**: An extension that provides AI-powered inline code completions by communicating with the Snek LSP server over stdio.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Server Lifecycle](#server-lifecycle)
3. [LSP Communication Protocol](#lsp-communication-protocol)
4. [Standard LSP Methods](#standard-lsp-methods)
5. [Custom Methods - Snek/Inline](#custom-methods---snekinline)
6. [File System Contracts](#file-system-contracts)
7. [Session Management](#session-management)
8. [Configuration & Environment](#configuration--environment)
9. [Error Handling](#error-handling)
10. [Testing Your Integration](#testing-your-integration)
11. [IDE-Specific Examples](#ide-specific-examples)

---

## Quick Start

### Minimal Integration Checklist

To integrate Snek LSP into your IDE, you need to:

1. ✅ Start the Snek LSP server as a child process
2. ✅ Communicate via JSON-RPC 2.0 over stdio
3. ✅ Implement standard LSP lifecycle methods
4. ✅ Call the custom `snek/inline` method for completions
5. ✅ (Optional) Provide UI for managing `.snek/` configuration files

### Prerequisites

- Snek LSP binary (`snek`) available in PATH or known location
- Environment variables configured:
  - `SNEK_API_KEY` (required)
  - `SNEK_API_BASE_URL` (optional, defaults to OpenAI)

---

## Server Lifecycle

### 1. Starting the Server

**Command**: `./snek` (or full path to binary)

**Transport**: stdio (stdin for requests, stdout for responses, stderr for logs)

**Example** (pseudo-code):
```javascript
const server = spawn('./snek', [], {
  stdio: ['pipe', 'pipe', 'pipe'],
  env: {
    ...process.env,
    SNEK_API_KEY: 'your-api-key',
    SNEK_API_BASE_URL: 'https://api.openai.com/v1'
  }
});
```

### 2. Initialization Sequence

```
Client → Server: initialize request
Server → Client: initialize response (with capabilities)
Client → Server: initialized notification
[Server is now ready to handle requests]
```

### 3. Shutdown Sequence

```
Client → Server: shutdown request
Server → Client: shutdown response
Client → Server: exit notification
[Server terminates]
```

---

## LSP Communication Protocol

### JSON-RPC 2.0 Format

All communication uses JSON-RPC 2.0 over stdio with content-length headers.

**Request Format**:
```
Content-Length: <byte-length>\r\n
\r\n
<JSON-RPC payload>
```

**Example Request**:
```
Content-Length: 246\r\n
\r\n
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "processId": 12345,
    "rootUri": "file:///path/to/workspace",
    "capabilities": {}
  }
}
```

**Response Format**:
```
Content-Length: <byte-length>\r\n
\r\n
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { ... }
}
```

**Error Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Error description"
  }
}
```

### Message Types

1. **Request**: Has `id`, expects response
2. **Response**: Has `id`, contains `result` or `error`
3. **Notification**: No `id`, no response expected

---

## Standard LSP Methods

### initialize

**Purpose**: Start LSP session and negotiate capabilities

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "processId": 12345,
    "rootUri": "file:///path/to/workspace",
    "capabilities": {
      "textDocument": {
        "synchronization": {
          "didOpen": true,
          "didChange": true,
          "didClose": true
        }
      }
    },
    "initializationOptions": null
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "capabilities": {
      "textDocumentSync": 1
    },
    "serverInfo": {
      "name": "snek-lsp",
      "version": "0.1.0"
    }
  }
}
```

**Notes**:
- `textDocumentSync: 1` means full document sync (not incremental)
- The server will create `.snek/` directory in the workspace root on initialization

---

### initialized

**Purpose**: Confirm initialization complete

**Notification** (no response):
```json
{
  "jsonrpc": "2.0",
  "method": "initialized",
  "params": {}
}
```

---

### textDocument/didOpen

**Purpose**: Notify server that a document was opened

**Notification**:
```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/didOpen",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/file.rs",
      "languageId": "rust",
      "version": 1,
      "text": "fn main() {\n    // code here\n}"
    }
  }
}
```

**Language IDs**: Use standard LSP language identifiers:
- `rust`, `python`, `javascript`, `typescript`, `java`, `go`, `cpp`, `c`, etc.

---

### textDocument/didChange

**Purpose**: Notify server that document content changed

**Notification**:
```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/didChange",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/file.rs",
      "version": 2
    },
    "contentChanges": [
      {
        "text": "fn main() {\n    println!(\"Hello\");\n}"
      }
    ]
  }
}
```

**Note**: Snek LSP uses **full sync mode** - send the entire document text in `contentChanges[0].text`.

---

### textDocument/didClose

**Purpose**: Notify server that document was closed

**Notification**:
```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/didClose",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/file.rs"
    }
  }
}
```

---

### shutdown

**Purpose**: Request server shutdown

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "method": "shutdown",
  "params": null
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "result": null
}
```

---

### exit

**Purpose**: Terminate server process

**Notification**:
```json
{
  "jsonrpc": "2.0",
  "method": "exit",
  "params": null
}
```

---

## Custom Methods - snek/inline

### Overview

The `snek/inline` method is the **core feature** of Snek LSP. It provides AI-powered code completions based on:
- Current cursor position (prefix/suffix)
- Chat history from `chat.json` (team conventions, previous conversations)
- Code contexts from `context.json` (related files)
- Session configuration from `session.json` (token limits)

### Request Format

**Method**: `snek/inline`

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "method": "snek/inline",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/file.rs"
    },
    "position": {
      "line": 10,
      "character": 15
    }
  }
}
```

**Parameters**:
- `textDocument.uri`: File URI (must be previously opened with `didOpen`)
- `position.line`: 0-indexed line number
- `position.character`: 0-indexed character offset in line

### Response Format

**Success Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "result": {
    "completion": "    let result = calculate_sum(a, b);\n    println!(\"Result: {}\", result);"
  }
}
```

**Response Fields**:
- `completion`: String containing the AI-generated code to insert at cursor position

### Error Responses

**Document Not Found**:
```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "error": {
    "code": -32602,
    "message": "Document not found or position invalid"
  }
}
```

**Internal Error** (e.g., API failure):
```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "error": {
    "code": -32603,
    "message": "Internal error"
  }
}
```

### Timing and Performance

- **Expected Latency**: 500ms - 2000ms (depends on AI API response time)
- **Recommendation**: Show loading indicator to user during request
- **Debouncing**: Recommended to debounce user typing (300-500ms) before triggering

### Example Integration Flow

```javascript
// 1. User types and pauses
onUserPause(async () => {
  const position = editor.getCursorPosition();
  const uri = editor.getDocumentUri();
  
  // 2. Send snek/inline request
  const response = await lspClient.sendRequest('snek/inline', {
    textDocument: { uri },
    position: { line: position.line, character: position.character }
  });
  
  // 3. Display completion as inline suggestion
  if (response.completion) {
    editor.showInlineSuggestion(response.completion);
  }
});
```

---

## File System Contracts

### Overview

Snek LSP stores all configuration in a `.snek/` directory in the workspace root. The server automatically:
- Creates `.snek/` on first run
- Generates a default session with UUID
- Watches all files for changes (auto-reload within 250ms)

### Directory Structure

```
.snek/
├── active.json              # Points to active session
└── sessions/
    └── <session-uuid>/
        ├── session.json     # Session metadata and limits
        ├── chat.json        # Chat history (conversation context)
        └── context.json     # Code contexts from other files
```

### File: active.json

**Purpose**: Specifies which session is currently active

**Location**: `.snek/active.json`

**Schema**:
```json
{
  "schema": 1,
  "id": "f64fd03f-dbf5-4f43-aa5d-c919f92cc419",
  "path": "sessions/f64fd03f-dbf5-4f43-aa5d-c919f92cc419"
}
```

**Fields**:
- `schema` (integer): Schema version (currently 1)
- `id` (string): UUID of active session
- `path` (string): Relative path to session directory

**Modifications**:
- LSP watches this file
- Changing it switches the active session (reload < 300ms)

---

### File: session.json

**Purpose**: Session metadata and configuration

**Location**: `.snek/sessions/<uuid>/session.json`

**Schema**:
```json
{
  "schema": 1,
  "id": "f64fd03f-dbf5-4f43-aa5d-c919f92cc419",
  "name": "default",
  "version": 0,
  "limits": {
    "max_tokens": 1600
  },
  "updated_at": "2025-11-03T12:34:56Z"
}
```

**Fields**:
- `schema` (integer): Schema version
- `id` (string): Session UUID
- `name` (string): Human-readable session name
- `version` (integer): Incremented on each modification (for cache invalidation)
- `limits.max_tokens` (integer): Maximum tokens for AI completion
- `updated_at` (string): ISO 8601 timestamp

**Modifications**:
- LSP watches this file
- Changes trigger full session reload

---

### File: chat.json

**Purpose**: Conversation history that provides context to AI

**Location**: `.snek/sessions/<uuid>/chat.json`

**Schema**:
```json
{
  "schema": 1,
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful coding assistant. Use snake_case for Python variables."
    },
    {
      "role": "user",
      "content": "How should I structure error handling in this project?"
    },
    {
      "role": "assistant",
      "content": "For this project, use explicit Result types and avoid unwrap() in production code..."
    }
  ]
}
```

**Fields**:
- `schema` (integer): Schema version
- `messages` (array): Ordered list of chat messages

**Message Format**:
- `role` (string): One of `"system"`, `"user"`, or `"assistant"`
- `content` (string): Message text

**Use Cases**:
1. **Team Conventions**: Add system messages with coding standards
2. **Project Context**: Include user/assistant exchanges about architecture decisions
3. **Style Guidance**: Specify naming conventions, formatting preferences

**Modifications**:
- LSP watches this file
- Changes applied to next completion request (< 250ms)
- Messages are included in AI prompt in order

**Extension UI Suggestions**:
- Provide UI to view/edit chat history
- Allow importing conversations from AI tools (ChatGPT, Claude, etc.)
- Quick actions to add common conventions

---

### File: context.json

**Purpose**: Code snippets from other files to include in AI context

**Location**: `.snek/sessions/<uuid>/context.json`

**Schema**:
```json
{
  "schema": 1,
  "contexts": [
    {
      "uri": "file:///path/to/project/src/models/user.rs",
      "start_line": 0,
      "end_line": 50,
      "language_id": "rust",
      "code": "// Auto-populated by LSP",
      "description": "User model definition",
      "last_modified": "2025-11-03T12:34:56Z"
    },
    {
      "uri": "file:///path/to/project/src/utils/validation.py",
      "start_line": 10,
      "end_line": 30,
      "language_id": "python",
      "code": "// Auto-populated by LSP",
      "description": "Email validation function",
      "last_modified": "2025-11-03T12:35:00Z"
    }
  ]
}
```

**Fields**:
- `schema` (integer): Schema version
- `contexts` (array): List of code contexts

**Context Object**:
- `uri` (string): **Required** - File URI (must be absolute `file://` URI)
- `start_line` (integer): **Required** - First line to include (0-indexed)
- `end_line` (integer): **Required** - Last line to include (exclusive)
- `language_id` (string): **Required** - Language identifier
- `code` (string): **Auto-populated** - Actual code content (extracted by LSP)
- `description` (string, optional): Human-readable description
- `last_modified` (string): **Auto-populated** - ISO 8601 timestamp

**Behavior**:
1. Extension adds context entries with `uri`, `start_line`, `end_line`, `language_id`
2. LSP reads the specified file and extracts lines
3. LSP populates `code` field automatically
4. LSP watches the source file for changes
5. When source file changes, LSP updates `code` and `last_modified` automatically

**Use Cases**:
- Reference model definitions when writing API handlers
- Include utility functions when writing business logic
- Show related test cases when writing new tests

**Extension UI Suggestions**:
- "Add to Context" action in file explorer or editor
- Visual indicator showing which files are in context
- Context panel to view/manage contexts
- Smart suggestions (e.g., "Add related files")

---

## Session Management

### Creating Sessions

Your extension can create new sessions by:

1. Creating directory: `.snek/sessions/<new-uuid>/`
2. Creating required files with valid schemas
3. Updating `active.json` to point to new session

**Example**:
```javascript
const sessionId = generateUUID();
const sessionDir = `.snek/sessions/${sessionId}`;

// Create directory
fs.mkdirSync(sessionDir, { recursive: true });

// Create session.json
fs.writeFileSync(`${sessionDir}/session.json`, JSON.stringify({
  schema: 1,
  id: sessionId,
  name: "My Custom Session",
  version: 0,
  limits: { max_tokens: 1600 },
  updated_at: new Date().toISOString()
}, null, 2));

// Create empty chat.json
fs.writeFileSync(`${sessionDir}/chat.json`, JSON.stringify({
  schema: 1,
  messages: []
}, null, 2));

// Create empty context.json
fs.writeFileSync(`${sessionDir}/context.json`, JSON.stringify({
  schema: 1,
  contexts: []
}, null, 2));

// Switch to new session
fs.writeFileSync('.snek/active.json', JSON.stringify({
  schema: 1,
  id: sessionId,
  path: `sessions/${sessionId}`
}, null, 2));
```

### Switching Sessions

To switch sessions, update `active.json`:

```javascript
fs.writeFileSync('.snek/active.json', JSON.stringify({
  schema: 1,
  id: targetSessionId,
  path: `sessions/${targetSessionId}`
}, null, 2));
```

LSP will detect the change and reload within 300ms.

### Session Use Cases

- **Per-Branch Sessions**: Different context for different feature branches
- **Per-Project Sessions**: Team conventions specific to different projects
- **Personal vs Team**: Personal coding preferences vs team standards
- **Experimentation**: Test different prompting strategies

---

## Configuration & Environment

### Required Environment Variables

**SNEK_API_KEY**:
- **Purpose**: API key for OpenAI-compatible endpoint
- **Required**: Yes
- **Example**: `export SNEK_API_KEY="sk-..."`

**SNEK_API_BASE_URL**:
- **Purpose**: Base URL for API endpoint
- **Required**: No
- **Default**: `https://api.openai.com/v1`
- **Example**: `export SNEK_API_BASE_URL="https://api.openai.com/v1"`

### Extension Configuration Options

Recommended user-facing settings for your extension:

```json
{
  "snek.serverPath": "/path/to/snek",
  "snek.apiKey": "sk-...",
  "snek.apiBaseUrl": "https://api.openai.com/v1",
  "snek.autoTrigger": true,
  "snek.debounceMs": 500,
  "snek.showInlinePreview": true
}
```

---

## Error Handling

### Server Logs

**Location**: stderr

**Format**: Lines prefixed with `[SNEK]`

**Example**:
```
[SNEK] Starting Snek Language Server...
[SNEK] Initializing workspace...
[SNEK] Workspace root: /path/to/project/.snek
[SNEK] Active session: /path/to/project/.snek/sessions/abc-123
[SNEK] Loaded session: abc-123 (version 0)
[SNEK] Starting file watcher...
[SNEK] Model API base: https://api.openai.com/v1
[SNEK] Server ready, listening on stdio...
```

**Error Examples**:
```
[SNEK] Failed to reload snapshot: Failed to read session.json
[SNEK] Model error: API request failed: 401 Unauthorized
[SNEK] Failed to update context file:///path/to/file.rs: File not found
```

### Common Errors

**1. SNEK_API_KEY Not Set**
```
Error: SNEK_API_KEY environment variable not set
```
**Solution**: Ensure environment variable is set when spawning server

**2. Invalid API Key**
```
[SNEK] Model error: ... 401 Unauthorized
```
**Solution**: Verify API key is valid

**3. Document Not Found**
```
error: { code: -32602, message: "Document not found or position invalid" }
```
**Solution**: Ensure `didOpen` was called before `snek/inline`

**4. File Watcher Failed**
```
[SNEK] Failed to watch active.json: Permission denied
```
**Solution**: Check file permissions on `.snek/` directory

### Error Recovery

**Server Crash**:
- Extension should detect server process exit
- Show error notification to user
- Offer to restart server
- Check stderr for crash reason

**API Timeout**:
- `snek/inline` may take up to 2 seconds
- Set request timeout to 5 seconds
- Show timeout error to user
- Log stderr for debugging

---

## Testing Your Integration

### 1. Smoke Test

```bash
# Verify server starts
export SNEK_API_KEY="test-key"
./snek &
SERVER_PID=$!
sleep 2
kill $SERVER_PID

# Check .snek/ was created
ls -la .snek/
```

### 2. Initialize Test

Send initialize request and verify response:

```javascript
const initRequest = {
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: {
    processId: process.pid,
    rootUri: "file:///test/workspace",
    capabilities: {}
  }
};

// Expected response should have textDocumentSync: 1
```

### 3. Document Sync Test

```javascript
// 1. Send didOpen
await client.sendNotification('textDocument/didOpen', {
  textDocument: {
    uri: 'file:///test/file.rs',
    languageId: 'rust',
    version: 1,
    text: 'fn main() {\n    \n}'
  }
});

// 2. Request completion
const response = await client.sendRequest('snek/inline', {
  textDocument: { uri: 'file:///test/file.rs' },
  position: { line: 1, character: 4 }
});

// 3. Verify response has completion field
assert(response.completion !== undefined);
```

### 4. File Watching Test

```javascript
// 1. Start server and wait for initialization
// 2. Modify .snek/sessions/<id>/chat.json
// 3. Wait 300ms
// 4. Request completion
// 5. Verify new chat context is applied
```

### 5. Real API Test

Set valid `SNEK_API_KEY` and test with actual AI endpoint:

```javascript
// Request completion with real context
const response = await client.sendRequest('snek/inline', {
  textDocument: { uri: 'file:///real/file.py' },
  position: { line: 10, character: 0 }
});

// Should get meaningful completion within 2s
console.log('AI Completion:', response.completion);
```

---

## IDE-Specific Examples

### VS Code

**Extension Structure**:
```typescript
// extension.ts
import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
  const serverPath = '/path/to/snek';
  
  const client = new LanguageClient(
    'snek-lsp',
    'Snek LSP',
    {
      command: serverPath,
      args: [],
      options: {
        env: {
          ...process.env,
          SNEK_API_KEY: config.get('apiKey'),
          SNEK_API_BASE_URL: config.get('apiBaseUrl')
        }
      }
    },
    {
      documentSelector: [{ scheme: 'file', language: '*' }],
      synchronize: {
        fileEvents: vscode.workspace.createFileSystemWatcher('**/*')
      }
    }
  );
  
  // Register custom snek/inline handler
  client.onReady().then(() => {
    vscode.commands.registerCommand('snek.inlineComplete', async () => {
      const editor = vscode.window.activeTextEditor;
      const position = editor.selection.active;
      
      const response = await client.sendRequest('snek/inline', {
        textDocument: { uri: editor.document.uri.toString() },
        position: { line: position.line, character: position.character }
      });
      
      // Show inline suggestion
      showInlineSuggestion(response.completion);
    });
  });
  
  client.start();
}
```

---

### Neovim

**Using nvim-lspconfig**:
```lua
-- lua/snek-lsp.lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define Snek LSP config
configs.snek = {
  default_config = {
    cmd = { '/path/to/snek' },
    filetypes = { '*' },
    root_dir = lspconfig.util.root_pattern('.git', '.snek'),
    settings = {},
  },
}

-- Setup Snek LSP
lspconfig.snek.setup{
  cmd_env = {
    SNEK_API_KEY = os.getenv('SNEK_API_KEY'),
    SNEK_API_BASE_URL = 'https://api.openai.com/v1',
  },
  on_attach = function(client, bufnr)
    -- Register snek/inline command
    vim.api.nvim_buf_create_user_command(bufnr, 'SnekComplete', function()
      local params = {
        textDocument = vim.lsp.util.make_text_document_params(),
        position = vim.lsp.util.make_position_params().position,
      }
      
      client.request('snek/inline', params, function(err, result)
        if result and result.completion then
          -- Insert completion at cursor
          vim.api.nvim_put({result.completion}, 'c', true, true)
        end
      end)
    end, {})
  end,
}
```

---

### IntelliJ IDEA

**Using LSP4IJ plugin**:
```kotlin
// SnekLspServer.kt
class SnekLspServerDescriptor : LanguageServerFactory {
    override fun createConnectionProvider(project: Project): StreamConnectionProvider {
        return ProcessStreamConnectionProvider(
            listOf("/path/to/snek"),
            project.basePath,
            mapOf(
                "SNEK_API_KEY" to getApiKey(),
                "SNEK_API_BASE_URL" to getApiBaseUrl()
            )
        )
    }
    
    override fun getLanguageId() = "*"
}

// Register custom snek/inline method
class SnekInlineCompletionAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val editor = e.getData(CommonDataKeys.EDITOR) ?: return
        val project = e.project ?: return
        val offset = editor.caretModel.offset
        
        val server = LanguageServiceAccessor.getLanguageServers(project, "*")
            .firstOrNull { it.serverDefinition is SnekLspServerDescriptor }
        
        val params = mapOf(
            "textDocument" to mapOf("uri" to editor.virtualFile.url),
            "position" to editor.offsetToLogicalPosition(offset)
        )
        
        server?.sendRequest("snek/inline", params) { result ->
            val completion = result["completion"] as? String
            // Show inline hint with completion
        }
    }
}
```

---

### Sublime Text

**Using LSP package**:
```json
// LSP.sublime-settings
{
  "clients": {
    "snek": {
      "enabled": true,
      "command": ["/path/to/snek"],
      "env": {
        "SNEK_API_KEY": "${snek_api_key}",
        "SNEK_API_BASE_URL": "https://api.openai.com/v1"
      },
      "selector": "source",
      "schemes": ["file"]
    }
  }
}
```

```python
# snek_inline.py
import sublime
import sublime_plugin
from LSP.plugin import Request, Session

class SnekInlineCompletionCommand(sublime_plugin.TextCommand):
    def run(self, edit):
        session = Session.for_view(self.view, "snek")
        if not session:
            return
        
        point = self.view.sel()[0].begin()
        row, col = self.view.rowcol(point)
        
        params = {
            "textDocument": {"uri": self.view.file_name()},
            "position": {"line": row, "character": col}
        }
        
        request = Request("snek/inline", params)
        session.send_request(request, self.on_result)
    
    def on_result(self, result):
        if result and "completion" in result:
            # Show phantom with completion
            self.view.show_popup(result["completion"])
```

---

## Best Practices

### 1. Debouncing

Don't trigger `snek/inline` on every keystroke:
```javascript
let debounceTimer;
editor.onDidChangeText(() => {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    requestInlineCompletion();
  }, 500); // Wait 500ms after user stops typing
});
```

### 2. Loading States

Show user that completion is being generated:
```javascript
async function requestCompletion() {
  showLoadingIndicator();
  try {
    const result = await client.sendRequest('snek/inline', params);
    showCompletion(result.completion);
  } catch (error) {
    showError('Completion failed');
  } finally {
    hideLoadingIndicator();
  }
}
```

### 3. Caching

Cache completions for same position to avoid redundant API calls:
```javascript
const completionCache = new Map();
const cacheKey = `${uri}:${line}:${char}`;

if (completionCache.has(cacheKey)) {
  return completionCache.get(cacheKey);
}

const result = await requestCompletion();
completionCache.set(cacheKey, result);
```

### 4. Error Recovery

Handle server crashes gracefully:
```javascript
serverProcess.on('exit', (code) => {
  if (code !== 0) {
    showErrorNotification('Snek LSP crashed. Restart?');
    offerRestart();
  }
});
```

### 5. User Feedback

Provide visibility into what's happening:
- Status bar indicator (idle/thinking/error)
- Notification for API errors
- Command to view server logs
- UI to manage sessions and contexts

---

## Debugging Tips

### 1. Enable Verbose Logging

Capture stderr from server process:
```javascript
const server = spawn('./snek', [], {
  stdio: ['pipe', 'pipe', 'pipe']
});

server.stderr.on('data', (data) => {
  console.log('[SNEK LOG]', data.toString());
});
```

### 2. Log JSON-RPC Messages

Log all requests/responses:
```javascript
function sendRequest(method, params) {
  console.log('→', method, JSON.stringify(params, null, 2));
  const response = await actualSendRequest(method, params);
  console.log('←', method, JSON.stringify(response, null, 2));
  return response;
}
```

### 3. Validate File System State

Check `.snek/` structure:
```bash
tree .snek/
cat .snek/active.json
cat .snek/sessions/*/session.json
```

### 4. Test with curl

Test API endpoint separately:
```bash
curl -X POST $SNEK_API_BASE_URL/chat/completions \
  -H "Authorization: Bearer $SNEK_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4","messages":[{"role":"user","content":"test"}]}'
```

---

## Support and Resources

### Documentation

- **This Guide**: Complete integration reference
- **`IMPLEMENTATION_STATUS.md`**: Implementation details and architecture
- **`README.md`**: User-facing documentation
- **`specs/001-snek-lsp/`**: Full specification and contracts

### Testing

- **Smoke Test**: `./test_lsp.sh` - Verify server works
- **Unit Tests**: `cargo test` - Run server test suite

### Community

For questions, issues, or contributions:
- Report bugs with server logs and JSON-RPC traces
- Include extension name and version
- Provide minimal reproduction steps

---

## Appendix: JSON Schema Definitions

### active.json Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["schema", "id", "path"],
  "properties": {
    "schema": { "type": "integer", "const": 1 },
    "id": { "type": "string", "format": "uuid" },
    "path": { "type": "string", "pattern": "^sessions/.+$" }
  }
}
```

### session.json Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["schema", "id", "name", "version", "limits", "updated_at"],
  "properties": {
    "schema": { "type": "integer", "const": 1 },
    "id": { "type": "string", "format": "uuid" },
    "name": { "type": "string" },
    "version": { "type": "integer", "minimum": 0 },
    "limits": {
      "type": "object",
      "required": ["max_tokens"],
      "properties": {
        "max_tokens": { "type": "integer", "minimum": 1 }
      }
    },
    "updated_at": { "type": "string", "format": "date-time" }
  }
}
```

### chat.json Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["schema", "messages"],
  "properties": {
    "schema": { "type": "integer", "const": 1 },
    "messages": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["role", "content"],
        "properties": {
          "role": { "type": "string", "enum": ["system", "user", "assistant"] },
          "content": { "type": "string" }
        }
      }
    }
  }
}
```

### context.json Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["schema", "contexts"],
  "properties": {
    "schema": { "type": "integer", "const": 1 },
    "contexts": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["uri", "start_line", "end_line", "language_id", "code", "last_modified"],
        "properties": {
          "uri": { "type": "string", "format": "uri" },
          "start_line": { "type": "integer", "minimum": 0 },
          "end_line": { "type": "integer", "minimum": 0 },
          "language_id": { "type": "string" },
          "code": { "type": "string" },
          "description": { "type": "string" },
          "last_modified": { "type": "string", "format": "date-time" }
        }
      }
    }
  }
}
```

---

**End of Extension Integration Guide**

For questions or clarifications, refer to the source code in the repository or the detailed specification documents in `specs/001-snek-lsp/`.

