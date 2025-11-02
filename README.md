# Snek

An AI-powered tab completion system with Language Server Protocol (LSP) support for multiple IDEs.

## Features

- 🚀 **LSP Server** - Provides intelligent code completions via Language Server Protocol
- 🎯 **Ghost Text Suggestions** - Inline AI-powered code suggestions
- 🔌 **VS Code Extension** - Ready-to-use extension for Visual Studio Code
- 🌍 **Multi-Language Support** - Works with Rust, JavaScript, TypeScript, Python, Java, and more
- ⚡ **Fast & Lightweight** - Built with Rust for maximum performance

## Quick Start

### 1. Build the LSP Server

```bash
cargo build --release
```

The binary will be available at `target/release/snek`

### 2. Install the VS Code Extension

```bash
cd vscode-extension
npm install
npm run compile
```

Then press `F5` in VS Code to launch the Extension Development Host, or see [TESTING.md](TESTING.md) for detailed instructions.

## Usage

### As an LSP Server

```bash
# Start the LSP server (reads from stdin, writes to stdout)
./target/debug/snek --lsp
```

### As a CLI Tool

```bash
# Show help
cargo run -- --help

# Run with a name
cargo run -- --name Alice

# Run the main command
cargo run -- run

# Run with a config file
cargo run -- run --config config.toml

# Show info
cargo run -- info

# Enable debug mode
cargo run -- -d run
cargo run -- -dd run  # level 2
```

## Development

```bash
# Build
cargo build

# Run
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Project Structure

```
snek/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library exports
│   ├── lsp/
│   │   ├── mod.rs        # LSP module exports
│   │   ├── backend.rs    # LSP implementation
│   │   └── server.rs     # LSP server setup
│   ├── ai/               # AI integration (future)
│   ├── indexing/         # Code indexing (future)
│   └── search/           # Code search (future)
├── vscode-extension/     # VS Code extension
│   ├── src/
│   │   └── extension.ts  # Extension entry point
│   ├── package.json      # Extension manifest
│   └── README.md         # Extension documentation
└── TESTING.md            # Testing guide
```

## Testing

See [TESTING.md](TESTING.md) for comprehensive testing instructions.

Quick test:

1. Build the server: `cargo build`
2. Open `vscode-extension` folder in VS Code
3. Press `F5` to launch Extension Development Host
4. Create a new file and trigger completions with `Ctrl+Space` / `Cmd+Space`

## Dependencies

### Rust Dependencies
- `clap` - Command line argument parsing
- `anyhow` - Error handling
- `colored` - Terminal colors
- `tower-lsp` - Language Server Protocol implementation
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization

### Extension Dependencies
- `vscode-languageclient` - VS Code LSP client
- `typescript` - TypeScript compiler

## Extending the LSP

The current implementation provides dummy completions. To integrate with AI models:

1. **Modify `src/lsp/backend.rs`**: Update the `completion` method to call your AI model
2. **Add context**: Use `params.text_document_position` to get cursor position and file context
3. **Integrate AI APIs**: Add API clients for OpenAI, Anthropic, or local models
4. **Implement caching**: Cache responses for better performance

Example modification in `backend.rs`:

```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    // Get file content and cursor position
    let position = params.text_document_position.position;
    
    // Call your AI model here
    let ai_suggestion = call_ai_model(file_content, position).await?;
    
    // Return the suggestion
    let item = CompletionItem {
        label: "AI Suggestion".to_string(),
        insert_text: Some(ai_suggestion),
        // ... other fields
    };
    
    Ok(Some(CompletionResponse::Array(vec![item])))
}
```

## IDE Support

Currently supported:
- ✅ Visual Studio Code

Coming soon:
- ⏳ Zed
- ⏳ Neovim
- ⏳ Sublime Text
- ⏳ IntelliJ IDEA (via LSP plugin)

## License

MIT or Apache-2.0
