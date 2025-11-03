# Snek Language Server Protocol

**Version**: 0.1.0  
**Status**: ✅ Core Implementation Complete

A Language Server Protocol (LSP) implementation that provides AI-powered code completion with team style guidance and project-aware context.

---

## Features

### 🎯 Core Capabilities

- **AI-Powered Code Completion**: Custom `snek/inline` LSP method for intelligent code suggestions
- **Team Style Guidance**: Configure coding conventions via chat history that influence all suggestions
- **Project-Aware Context**: Reference code from other files for contextual completions
- **Session Management**: Switch between different sessions with isolated chat histories and contexts
- **Real-Time Updates**: File watching with automatic reload when configurations change
- **OpenAI Compatible**: Works with any OpenAI-compatible API endpoint

### 🏗️ Architecture

- **Filesystem-Based Communication**: All configuration via JSON files in `.snek/` directory
- **RAM Hot Path**: In-memory snapshot for fast completion requests
- **Atomic Operations**: Safe file operations with tmp-file-then-rename pattern
- **Debounced Watching**: 200ms debounce for efficient file change detection
- **Dual-Mode Watching**: Monitors both session files and individual context files

---

## Quick Start

### Prerequisites

- Rust 1.70+ (2024 edition)
- An OpenAI-compatible API endpoint and key

### Installation

1. **Clone and build**:
   ```bash
   git clone <repository-url>
   cd snek_lsp
   cargo build --release
   ```

2. **Set environment variables**:
   ```bash
   export SNEK_API_KEY="your-api-key-here"
   export SNEK_API_BASE_URL="https://api.openai.com/v1"  # Optional, defaults to OpenAI
   ```

3. **Run the server**:
   ```bash
   ./target/release/snek
   ```

The server listens on stdio for LSP communication.

---

## Configuration

### Workspace Structure

The LSP automatically creates and manages a `.snek/` directory in your project root:

```
.snek/
├── active.json              # Points to the active session
└── sessions/
    └── {session-id}/
        ├── session.json     # Session metadata and limits
        ├── chat.json        # Chat history (team conventions)
        └── context.json     # Code contexts from other files
```

### File Formats

#### `active.json`
```json
{
  "schema": 1,
  "id": "uuid-here",
  "path": "sessions/uuid-here"
}
```

#### `session.json`
```json
{
  "schema": 1,
  "id": "uuid-here",
  "name": "default",
  "version": 0,
  "limits": {
    "max_tokens": 1600
  },
  "updated_at": "2025-11-03T00:00:00Z"
}
```

#### `chat.json`
```json
{
  "schema": 1,
  "messages": [
    {
      "role": "system",
      "content": "Use snake_case for variable names"
    },
    {
      "role": "user",
      "content": "Prefer explicit error handling over unwrap()"
    }
  ]
}
```

#### `context.json`
```json
{
  "schema": 1,
  "contexts": [
    {
      "uri": "file:///path/to/file.rs",
      "start_line": 0,
      "end_line": 50,
      "language_id": "rust",
      "code": "// code will be auto-populated",
      "description": "User model definition",
      "last_modified": "2025-11-03T00:00:00Z"
    }
  ]
}
```

---

## Usage

### Basic Completion

1. Open a file in your LSP-compatible editor (VS Code, Neovim, etc.)
2. Position your cursor where you want a completion
3. Trigger the custom `snek/inline` method
4. Receive AI-generated code suggestion within 2 seconds

### Team Style Guidance

1. Edit `.snek/sessions/{id}/chat.json`
2. Add style preferences as system or user messages
3. Save the file (LSP auto-reloads within 250ms)
4. Future completions will respect these conventions

### Code Context

1. Edit `.snek/sessions/{id}/context.json`
2. Add file references with line ranges
3. Save the file (LSP auto-reloads and watches the referenced files)
4. Completions will be aware of the referenced code

### Session Switching

1. Create a new session directory in `.snek/sessions/`
2. Update `.snek/active.json` to point to the new session
3. Save the file (LSP switches sessions within 300ms)

---

## Development

### Running Tests

```bash
# All tests
cargo test

# Specific test suite
cargo test session_io
cargo test document_store

# Release mode
cargo test --release
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Apply clippy fixes
cargo clippy --fix --allow-dirty

# Check compilation
cargo check
```

### Project Structure

```
snek_lsp/
├── src/
│   ├── lib.rs                 # Module exports
│   ├── main.rs                # Entry point
│   ├── snapshot.rs            # Core data structures
│   ├── session_io.rs          # File I/O operations
│   ├── watcher.rs             # File system watching
│   ├── document_store.rs      # Document content tracking
│   ├── model.rs               # AI model integration
│   └── lsp/
│       ├── mod.rs             # LSP module exports
│       ├── backend.rs         # LSP implementation
│       └── server.rs          # Server setup
├── tests/
│   ├── session_io_test.rs     # Session I/O tests
│   └── document_store_test.rs # Document tracking tests
├── Cargo.toml                 # Dependencies
└── README.md                  # This file
```

---

## Testing Status

- ✅ **Unit Tests**: 9/9 passing
  - Session I/O: 4 tests
  - Document Store: 5 tests
- ✅ **Build**: Clean compilation in debug and release modes
- ✅ **Linter**: All clippy warnings resolved
- ⏳ **Integration Tests**: Ready for implementation
- ⏳ **Manual Tests**: Ready for execution with real editors

---

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Completion Latency | <2s | ⏳ Needs validation with real API |
| File Change Detection | <300ms | ✅ Implemented (200ms debounce) |
| Memory Usage | <100MB | ⏳ Needs profiling |

---

## Known Limitations

1. **Single Document Tracking**: Only the active document is cached (by design)
2. **Full Text Sync**: Uses LSP full sync mode (not incremental)
3. **No Result Caching**: Every completion calls the AI model
4. **No Rate Limiting**: No built-in rate limiting for API calls
5. **No Retry Logic**: Failed API calls are not automatically retried

These are documented design decisions that can be enhanced in future versions.

---

## Troubleshooting

### LSP Not Starting

1. Check that `SNEK_API_KEY` is set:
   ```bash
   echo $SNEK_API_KEY
   ```

2. Verify the binary exists:
   ```bash
   ls -lh target/release/snek
   ```

3. Check LSP logs (stderr):
   ```bash
   ./target/release/snek 2> snek.log
   ```

### Completions Not Appearing

1. Verify `.snek/` directory was created
2. Check that `active.json` points to a valid session
3. Ensure the session directory has all required files
4. Look for `[SNEK]` prefixed messages in stderr

### File Changes Not Detected

1. Check file watcher is running (look for `[SNEK] Starting file watcher...` in logs)
2. Verify files are being saved (not just modified in editor buffer)
3. Wait 200-300ms after saving for debounce to complete

---

## Contributing

### Adding New Features

1. Create a new branch from `main`
2. Implement the feature with tests
3. Run `cargo test` and `cargo clippy`
4. Update documentation
5. Submit a pull request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all clippy warnings
- Add doc comments (`///`) for public APIs
- Use `[SNEK]` prefix for all log messages

---

## License

MIT or Apache-2.0

---

## Acknowledgments

Built with:
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - LSP framework
- [tokio](https://tokio.rs/) - Async runtime
- [notify](https://github.com/notify-rs/notify) - File watching
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [arc-swap](https://github.com/vorner/arc-swap) - Lock-free atomic updates

---

## See Also

- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) - Detailed implementation progress
- [TESTING.md](TESTING.md) - Comprehensive testing guide
- [specs/001-snek-lsp/](specs/001-snek-lsp/) - Complete specification and design documents
