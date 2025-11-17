#!/bin/bash
# Simple smoke test for Snek LSP

set -e

echo "=== Snek LSP Smoke Test ==="
echo ""

# Check if binary exists
if [ ! -f "target/release/snek" ]; then
    echo "❌ Binary not found. Run 'cargo build --release' first."
    exit 1
fi

echo "✅ Binary found: target/release/snek"
echo ""

# Create a test workspace
TEST_DIR=$(mktemp -d)
echo "📁 Created test workspace: $TEST_DIR"

# Set dummy API key for testing
export SNEK_API_KEY="test-key"
export SNEK_API_BASE_URL="https://api.openai.com/v1"

# Change to test directory
cd "$TEST_DIR"

# Create a simple test file
cat > test.rs <<'EOF'
fn main() {
    println!("Hello, world!");
}
EOF

echo "✅ Created test.rs"
echo ""

# Test: Run LSP and let it initialize .snek/ directory
echo "🚀 Testing LSP initialization..."
echo ""
echo "Starting LSP server (will run for 2 seconds)..."

# Run the LSP server in background, redirect stderr to capture logs
"$OLDPWD/target/release/snek" 2> lsp.log &
LSP_PID=$!

# Give it time to initialize
sleep 2

# Send SIGTERM to gracefully shut down
kill -TERM $LSP_PID 2>/dev/null || true
wait $LSP_PID 2>/dev/null || true

echo "Server stopped"
echo ""

# Check results
echo "=== Test Results ==="
echo ""

if [ -d ".snek" ]; then
    echo "✅ .snek/ directory created"
    echo ""
    echo "Directory structure:"
    find .snek -type f | sort
    echo ""
    
    if [ -f ".snek/active.json" ]; then
        echo "✅ active.json created"
        echo "Contents:"
        cat .snek/active.json | jq . 2>/dev/null || cat .snek/active.json
        echo ""
    else
        echo "❌ active.json not found"
    fi
    
    # Check for session directory
    SESSION_DIR=$(find .snek/sessions -type d -mindepth 1 -maxdepth 1 | head -1)
    if [ -n "$SESSION_DIR" ]; then
        echo "✅ Session directory created: $(basename $SESSION_DIR)"
        
        if [ -f "$SESSION_DIR/session.json" ]; then
            echo "✅ session.json created"
        fi
        
        if [ -f "$SESSION_DIR/chat.json" ]; then
            echo "✅ chat.json created"
        fi
        
        if [ -f "$SESSION_DIR/context.json" ]; then
            echo "✅ context.json created"
        fi
    else
        echo "❌ No session directory found"
    fi
else
    echo "❌ .snek/ directory not created"
    echo ""
    echo "Server logs:"
    cat lsp.log 2>/dev/null || echo "(no logs captured)"
fi

echo ""
echo "=== Cleanup ==="

# Return to original directory
cd "$OLDPWD"

# Cleanup
rm -rf "$TEST_DIR"
echo "✅ Cleaned up test workspace"

echo ""
echo "=== Test Complete ==="
