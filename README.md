<div align="center">

<img src="./assets/logo.png" alt="Snek Logo" width="200"/>

# Snek - Write code, not prompts

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![VS Code](https://img.shields.io/badge/VS%20Code-Extension-blue)](../snek_vscode)
[![Neovim](https://img.shields.io/badge/Neovim-Plugin-green)](../snek-nvim)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)

**Context-aware AI code completions powered by Cerebras - The fastest inference on the planet**

[Features](#-features) • [Installation](#-installation) • [Configuration](#%EF%B8%8F-configuration) • [How It Works](#-how-it-works) • [Contributing](#-contributing)

</div>

---

## Features

- **Blazing Fast** - Powered by Cerebras's ultra-low latency inference (1000+ tokens/sec)
- **Customizable Context** - User can customize the cotext that Snek gets
  - **Markdown Context** - Add project-specific documentation that the AI uses for better completions
  - **Code Context** - Add code URIs and line ranges for Snek to read any relevant code you have
- **Smart Sessions** - Organize different contexts for different tasks or features
- **Multi-Language** - Supports Rust, Python, JavaScript, TypeScript, Go, C/C++, Java, and Lua
- **LSP-Based** - Works with VSCode and Nvim
- **Open Source** - MIT Licensed, no telemetry, runs locally

## Why Cerebras?

Snek uses **Cerebras** exclusively for AI completions because of its unmatched throughput and latency:

- **1,000+ tokens/second** - 10x faster than traditional GPU inference
- **Sub-100ms latency** - Completions appear instantly as you type
- **Best UX** - No laggy, stuttering suggestions that break your flow

## Installation

### VSCode

1. **Install the extension:**
   ```bash
   # From the marketplace (coming soon)
   # Or install from VSIX
   code --install-extension snek-lsp-darwin-arm64-x.x.x.vsix  # Apple Silicon Mac
   code --install-extension snek-lsp-darwin-x64-x.x.x.vsix     # Intel Mac
   # No support for other targets for now
   ```

2. **Get a Cerebras API key:**
   - Visit [https://cloud.cerebras.ai/](https://cloud.cerebras.ai/)
   - Sign up and create an API key (free tier available!)


### Neovim

**Using [lazy.nvim](https://github.com/folke/lazy.nvim):**

```lua
{
  "4tyone/snek-nvim",
  config = function()
    require("snek-nvim").setup({
      api_key = "your-cerebras-api-key",
      model = "qwen-3-235b-a22b-instruct-2507",  -- Optional, this is the default
    })
  end,
}
```

**Using [packer.nvim](https://github.com/wbthomason/packer.nvim):**

```lua
use {
  "4tyone/snek-nvim",
  config = function()
    require("snek-nvim").setup({
      api_key = "your-cerebras-api-key",
      model = "qwen-3-235b-a22b-instruct-2507",
    })
  end,
}
```

See the [snek-nvim repository](https://github.com/4tyone/snek-nvim) for full installation and configuration details.

## Configuration

### VSCode Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `snek.apiKey` | `""` | Your Cerebras API key from https://cloud.cerebras.ai/ |
| `snek.model` | `qwen-3-235b-a22b-instruct-2507` | Model to use for completions |

**Recommended Models:**
- `qwen-3-235b-a22b-instruct-2507` (recommended - best quality/speed balance and used as a default model)
- `zai-glm-4.6` (smarter but more expensive)

### Neovim Configuration

**Full configuration example:**

```lua
require("snek-nvim").setup({
  -- Required: Your Cerebras API key
  api_key = "your-cerebras-api-key",

  -- Optional: Model to use (default shown)
  model = "qwen-3-235b-a22b-instruct-2507",

  -- Keymaps (defaults shown)
  keymaps = {
    accept_suggestion = "<Tab>",     -- Accept the full suggestion
    clear_suggestion = "<C-]>",      -- Dismiss the suggestion
    accept_word = "<C-j>",           -- Accept only the next word
  },

  -- Optional settings
  ignore_filetypes = { cpp = true }, -- Filetypes to disable completions
  disable_inline_completion = false, -- Disable ghost text suggestions
  disable_keymaps = false,           -- Don't set up default keymaps
  log_level = "info",                -- "off", "trace", "debug", "info", "warn", "error"

  -- Custom suggestion color (optional)
  color = {
    suggestion_color = "#808080",
    cterm = 244,
  },
})
```

**Neovim Commands:**
- `:SnekStart` - Start the Snek LSP
- `:SnekStop` - Stop the Snek LSP
- `:SnekRestart` - Restart the Snek LSP
- `:SnekToggle` - Toggle the Snek LSP on/off
- `:SnekStatus` - Show whether Snek is running
- `:SnekShowLog` - Open the log file
- `:SnekClearLog` - Clear the log file

## How It Works

Snek uses a unique **session-based context system** that makes AI completions actually useful:

### 1. Project Structure

When you open a project, Snek creates a `.snek/` directory:

```
your-project/
├── .snek/                      # Snek workspace
│   ├── active.json            # Current session pointer
│   ├── sessions/
│   │   └── {session-id}/
│   │       ├── session.json   # Session config (limits, version)
│   │       ├── context/       # Markdown context files
│   │       │   ├── architecture.md
│   │       │   ├── conventions.md
│   │       │   └── api-patterns.md
│   │       └── code_snippets.json  # Referenced code
│   ├── scripts/               # Session management scripts
│   └── commands/              # Custom slash commands
└── your-code/
```

### 2. Markdown Context Files

Add markdown files to `.snek/sessions/{id}/context/` to guide the AI. These files are **always** included in completion requests:

**Example: architecture.md**
```markdown
# Project Architecture

This is a microservices-based e-commerce platform:
- `api-gateway` - Routes requests to services
- `user-service` - Handles authentication and user profiles
- `product-service` - Manages product catalog and inventory

All services communicate via REST APIs with JSON payloads.
Database: PostgreSQL with SQLAlchemy ORM
```

**Example: conventions.md**
```markdown
# Coding Conventions

- **Variables**: Use snake_case
- **Functions**: All public functions must have docstrings
- **Async**: Prefer async/await over callbacks
- **Errors**: Use explicit error handling, avoid .unwrap()
- **Types**: Use TypeScript strict mode, no `any` types
```

Snek reads these files and uses them as context for **every** completion, ensuring generated code follows your project's patterns and conventions.

### 3. Code Snippets

Reference important code that Snek should know about. Add them to `code_snippets.json`:

```json
{
  "schema": 1,
  "snippets": [
    {
      "uri": "file:///path/to/project/src/utils/api.ts",
      "start_line": 0,
      "end_line": 50,
      "language_id": "typescript",
      "description": "API utility functions and standard error handling"
    },
    {
      "uri": "file:///path/to/project/src/models/user.py",
      "start_line": 10,
      "end_line": 45,
      "language_id": "python",
      "description": "User model with validation and database schema"
    }
  ]
}
```

Snek will:
- Watch these files for changes
- Include them in completion context
- Update automatically when they're modified

### 4. Session Management

Create different sessions for different tasks or features:

```bash
# Create a new session for a feature
.snek/scripts/new-session.sh "authentication-refactor"

# Switch between sessions
.snek/scripts/switch-session.sh a3f92a1c  # First 8 chars of session ID

```

Each session has its own:
- Context files (different conventions, architecture notes)
- Code snippets (different relevant files)
- Token limits (adjust based on complexity)

## Architecture

```
┌─────────────────────┐
│   Your Editor       │
│   (VSCode/Neovim)   │
└──────────┬──────────┘
           │ LSP Protocol (stdio)
           │
┌──────────▼──────────┐
│     Snek LSP        │
│                     │
│  ┌───────────────┐  │
│  │   Session     │  │
│  │   Manager     │  │
│  └───────────────┘  │
│  ┌───────────────┐  │
│  │   File        │  │
│  │   Watcher     │  │
│  └───────────────┘  │
│  ┌───────────────┐  │
│  │   Context     │  │
│  │   Builder     │  │
│  └───────────────┘  │
└──────────┬──────────┘
           │ HTTPS
           │
┌──────────▼──────────┐
│   Cerebras API      │
│   api.cerebras.ai   │
│                     │
│   1000+ tokens/sec  │
│   Sub-100ms latency │
└─────────────────────┘
```

### How a Completion Works

1. **User types code** in their editor
2. **Editor sends LSP request** (`snek/inline`) with cursor position
3. **Snek loads context**:
   - Current file prefix/suffix around cursor
   - Markdown context files from active session
   - Referenced code snippets
4. **Snek builds prompt** with all context
5. **Cerebras generates completion** in <800ms
6. **Snek extracts code** (removes markdown fences, explanations)
7. **Editor shows inline suggestion** to user

All in under 2 seconds from keypress to suggestion!

## Development

### Building from Source

```bash
# Clone the repo
git clone https://github.com/4tyone/snek.git
cd snek

# Build and package VSCode extension
./build_and_package.sh

# Install locally
code --install-extension ../snek_vscode/snek-lsp-darwin-arm64-x.x.x.vsix
```

### Project Structure

```
snek-lsp/
├── src/
│   ├── main.rs               # Entry point
│   ├── lsp/
│   │   ├── server.rs         # LSP server initialization
│   │   └── backend.rs        # LSP protocol implementation
│   ├── model.rs              # Cerebras API integration
│   ├── session_io.rs         # Session file I/O
│   ├── watcher.rs            # File system watching
│   ├── snapshot.rs           # In-memory context snapshots
│   └── document_store.rs     # Document content tracking
├── templates/                # Default scripts/commands
│   ├── scripts/
│   │   ├── new-session.sh
│   │   └── switch-session.sh
│   └── commands/
│       └── snek.share.md
└── build_and_package.sh      # Build script for all platforms
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test session_io
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check compilation without building
cargo check
```

## Contributing

We love contributions! Here's how you can help:

### Ways to Contribute

1. **Report bugs** - Open an issue with reproduction steps
2. **Request features** - Tell us what would make Snek better
3. **Submit PRs** - Fix bugs or add features
4. **Improve docs** - Help others understand and use Snek
5. **Share feedback** - Let us know how Snek works for you!

### Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cross-compilation targets (macOS only)
rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Run LSP locally (for testing)
cd ../snek_lsp
cargo run --release
```

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo fmt`
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to your fork (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## Privacy & Security

- **No telemetry** - Snek never sends usage data or analytics
- **Local processing** - All context stays on your machine
- **API security** - Your Cerebras API key is stored in editor settings (encrypted by the editor)
- **Open source** - Every line of code is public and auditable
- **No account required** - Just get a Cerebras API key and go

## Troubleshooting

### LSP not starting

**VSCode:**
- Open Output panel (View → Output)
- Select "Snek Language Server" from dropdown
- Check for error messages

**Neovim:**
```lua
:LspInfo  -- Check if Snek LSP is attached
:LspLog   -- View LSP logs
```

### No completions appearing

1. **Check API key is configured** - Open settings and verify `snek.apiKey` is set
2. **Check file type** - Snek only activates for supported languages
3. **Check network** - Ensure you can reach `api.cerebras.ai`
4. **Check session** - Verify `.snek/` directory exists with valid `active.json`

### "Bad CPU type" error (macOS)

Your binary architecture doesn't match your Mac:
- Intel Mac needs `darwin-x64` version
- Apple Silicon needs `darwin-arm64` version

Reinstall the correct version for your architecture.

## License

MIT License - see [LICENSE](./LICENSE) for details.

You're free to use Snek in personal and commercial projects, modify it, and distribute it.

## Acknowledgments

- **Cerebras** - For providing the fastest inference infrastructure on the planet
- **tower-lsp** - Excellent LSP framework for Rust
- **The Rust Community** - For creating an amazing ecosystem

## Community & Support

- **GitHub Issues** - [Report bugs or request features](https://github.com/4tyone/snek/issues)
- **GitHub Discussions** - [Ask questions and share ideas](https://github.com/4tyone/snek/discussions)

---

<div align="center">

**Built by developers, for developers**

[Star us on GitHub](https://github.com/4tyone/snek) | [Report a Bug](https://github.com/4tyone/snek/issues) | [Request a Feature](https://github.com/4tyone/snek/issues/new)

</div>
