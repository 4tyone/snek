#!/bin/bash

# Quick Start Script for Snek LSP
# This script builds the LSP server and sets up the VS Code extension

set -e

echo "🐍 Snek LSP Quick Start"
echo "======================="
echo

# Step 1: Build the LSP server
echo "📦 Step 1: Building LSP server..."
cargo build
if [ $? -eq 0 ]; then
    echo "✅ LSP server built successfully at target/debug/snek"
else
    echo "❌ Failed to build LSP server"
    exit 1
fi
echo

# Step 2: Set up VS Code extension
echo "📦 Step 2: Setting up VS Code extension..."
cd vscode-extension

if [ ! -d "node_modules" ]; then
    echo "Installing npm dependencies..."
    npm install
fi

echo "Compiling TypeScript..."
npm run compile

if [ $? -eq 0 ]; then
    echo "✅ Extension compiled successfully"
else
    echo "❌ Failed to compile extension"
    exit 1
fi
echo

# Step 3: Instructions
echo "✨ Setup complete!"
echo
echo "To test the extension:"
echo "1. Open VS Code in the vscode-extension folder:"
echo "   cd vscode-extension && code ."
echo
echo "2. Press F5 to launch the Extension Development Host"
echo
echo "3. In the new window, create a test file (e.g., test.rs)"
echo
echo "4. Trigger completions with Ctrl+Space (Cmd+Space on Mac)"
echo
echo "For detailed testing instructions, see TESTING.md"
echo
echo "🎉 Happy coding!"

