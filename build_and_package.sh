#!/usr/bin/env bash
#
# build_and_package.sh - Build LSP and package VSCode extension for all platforms
#
# This script:
# 1. Builds the LSP for macOS Intel (x86_64) and Apple Silicon (aarch64)
# 2. Copies binaries to the VSCode extension folder
# 3. Packages the extension for each platform
#

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VSCODE_EXT_DIR="$SCRIPT_DIR/../snek_vscode"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         Snek LSP Build & Package Script                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Step 1: Build LSP for macOS Intel
echo -e "${YELLOW}[1/7]${NC} Building LSP for macOS Intel (x86_64)..."
cd "$SCRIPT_DIR"
cargo build --release --target x86_64-apple-darwin
echo -e "${GREEN}✓${NC} Built x86_64 binary"
echo ""

# Step 2: Build LSP for macOS Apple Silicon
echo -e "${YELLOW}[2/7]${NC} Building LSP for macOS Apple Silicon (aarch64)..."
cargo build --release --target aarch64-apple-darwin
echo -e "${GREEN}✓${NC} Built aarch64 binary"
echo ""

# Step 3: Code sign binaries (ad-hoc signing for distribution)
echo -e "${YELLOW}[3/7]${NC} Code signing binaries..."
codesign --force --sign - target/x86_64-apple-darwin/release/snek
codesign --force --sign - target/aarch64-apple-darwin/release/snek
echo -e "${GREEN}✓${NC} Signed both binaries (ad-hoc)"
echo ""

# Step 4: Package darwin-x64
echo -e "${YELLOW}[4/7]${NC} Copying x86_64 binary and packaging darwin-x64 extension..."
cp target/x86_64-apple-darwin/release/snek "$VSCODE_EXT_DIR/snek"
cp target/x86_64-apple-darwin/release/snek "$VSCODE_EXT_DIR/bin/snek"
cd "$VSCODE_EXT_DIR"
npx vsce package --target darwin-x64 > /dev/null 2>&1
echo -e "${GREEN}✓${NC} Packaged darwin-x64 extension"
echo ""

# Step 5: Package darwin-arm64
echo -e "${YELLOW}[5/7]${NC} Copying aarch64 binary and packaging darwin-arm64 extension..."
cp "$SCRIPT_DIR/target/aarch64-apple-darwin/release/snek" "$VSCODE_EXT_DIR/snek"
cp "$SCRIPT_DIR/target/aarch64-apple-darwin/release/snek" "$VSCODE_EXT_DIR/bin/snek"
npx vsce package --target darwin-arm64 > /dev/null 2>&1
echo -e "${GREEN}✓${NC} Packaged darwin-arm64 extension"
echo ""

# Step 6: Display results
echo -e "${YELLOW}[6/7]${NC} Verifying binaries..."
cd "$SCRIPT_DIR"
echo "  x86_64: $(file target/x86_64-apple-darwin/release/snek | cut -d: -f2)"
echo "  aarch64: $(file target/aarch64-apple-darwin/release/snek | cut -d: -f2)"
echo ""

# Step 7: Show packaged files
echo -e "${YELLOW}[7/7]${NC} Packaged extensions:"
ls -lh "$VSCODE_EXT_DIR"/*.vsix | grep -E "darwin-(x64|arm64)" | awk '{printf "  %s (%s)\n", $9, $5}'
echo ""

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                    ✓ Build Complete!                          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Install the extension:"
echo -e "  ${BLUE}Intel Mac:${NC}"
echo "    code --install-extension $VSCODE_EXT_DIR/snek-lsp-darwin-x64-0.1.0.vsix"
echo ""
echo -e "  ${BLUE}Apple Silicon Mac:${NC}"
echo "    code --install-extension $VSCODE_EXT_DIR/snek-lsp-darwin-arm64-0.1.0.vsix"
echo ""
