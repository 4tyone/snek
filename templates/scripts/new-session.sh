#!/usr/bin/env bash
#
# new-session.sh - Create a new snek session with proper structure
#
# Usage:
#   ./new-session.sh [session_name]
#
# If session_name is not provided, defaults to "default"
#

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the script's directory and navigate to .snek root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

# Parse arguments
SESSION_NAME="${1:-default}"

# Generate UUID
# Try different UUID generation methods for portability
if command -v uuidgen &> /dev/null; then
    # macOS and some Linux systems
    SESSION_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
elif command -v uuid &> /dev/null; then
    # Some Linux systems
    SESSION_ID=$(uuid -v4)
elif command -v python3 &> /dev/null; then
    # Fallback to Python
    SESSION_ID=$(python3 -c "import uuid; print(str(uuid.uuid4()))")
else
    echo "Error: No UUID generator found. Please install uuidgen, uuid, or python3."
    exit 1
fi

echo -e "${BLUE}Creating new snek session...${NC}"
echo -e "Session ID: ${GREEN}$SESSION_ID${NC}"
echo -e "Session Name: ${GREEN}$SESSION_NAME${NC}"

# Create session directory structure
SESSION_DIR="sessions/$SESSION_ID"
mkdir -p "$SESSION_DIR/context"

echo -e "${BLUE}✓${NC} Created directory structure: $SESSION_DIR/"

# Get current timestamp in RFC3339 format
if command -v python3 &> /dev/null; then
    TIMESTAMP=$(python3 -c "from datetime import datetime, timezone; print(datetime.now(timezone.utc).isoformat())")
else
    # Fallback to date command (may vary by system)
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%S.%6N%z")
fi

# Create session.json
cat > "$SESSION_DIR/session.json" <<EOF
{
  "id": "$SESSION_ID",
  "limits": {
    "max_tokens": 1600
  },
  "name": "$SESSION_NAME",
  "schema": 1,
  "updated_at": "$TIMESTAMP",
  "version": 0
}
EOF

echo -e "${BLUE}✓${NC} Created session.json"

# Create code_snippets.json
cat > "$SESSION_DIR/code_snippets.json" <<EOF
{
  "schema": 1,
  "snippets": []
}
EOF

echo -e "${BLUE}✓${NC} Created code_snippets.json"

# Update active.json to point to new session
cat > "active.json" <<EOF
{
  "id": "$SESSION_ID",
  "path": "sessions/$SESSION_ID",
  "schema": 1
}
EOF

echo -e "${BLUE}✓${NC} Updated active.json"

# Print summary
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ Session created successfully!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Session details:"
echo "  ID:        $SESSION_ID"
echo "  Name:      $SESSION_NAME"
echo "  Location:  .snek/$SESSION_DIR/"
echo ""
echo "Structure:"
echo "  .snek/$SESSION_DIR/"
echo "  ├── session.json"
echo "  ├── code_snippets.json"
echo "  └── context/"
echo ""
echo "Next steps:"
echo "  1. Add markdown context files to: .snek/$SESSION_DIR/context/"
echo "  2. Add code snippets to: .snek/$SESSION_DIR/code_snippets.json"
echo "  3. Your snek LSP will automatically pick up this new active session"
echo ""
