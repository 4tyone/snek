#!/usr/bin/env bash
#
# switch-session.sh - Switch the active snek session
#
# Usage:
#   ./switch-session.sh <session_id_prefix>
#
# The session_id_prefix can be:
#   - Full UUID: aaf82595-38b4-4aef-a2c0-f7b4c2ffabae
#   - First 8 chars: aaf82595
#

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get the script's directory and navigate to .snek root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

# Check if argument provided
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: Session ID prefix required${NC}"
    echo ""
    echo "Usage: $0 <session_id_prefix>"
    echo ""
    echo "Available sessions:"
    for session_dir in sessions/*/; do
        if [ -d "$session_dir" ]; then
            session_id=$(basename "$session_dir")
            session_name="unknown"

            if [ -f "$session_dir/session.json" ]; then
                if command -v python3 &> /dev/null; then
                    session_name=$(python3 -c "import json; print(json.load(open('$session_dir/session.json')).get('name', 'unknown'))" 2>/dev/null || echo "unknown")
                elif command -v jq &> /dev/null; then
                    session_name=$(jq -r '.name // "unknown"' "$session_dir/session.json" 2>/dev/null || echo "unknown")
                fi
            fi

            echo "  ${session_id:0:8} - $session_name ($session_id)"
        fi
    done
    exit 1
fi

SESSION_PREFIX="$1"

# Find matching session
MATCHED_SESSION=""
MATCH_COUNT=0

for session_dir in sessions/*/; do
    if [ -d "$session_dir" ]; then
        session_id=$(basename "$session_dir")

        # Check if session_id starts with the prefix
        if [[ "$session_id" == "$SESSION_PREFIX"* ]]; then
            MATCHED_SESSION="$session_id"
            MATCH_COUNT=$((MATCH_COUNT + 1))
        fi
    fi
done

# Validate matches
if [ $MATCH_COUNT -eq 0 ]; then
    echo -e "${RED}Error: No session found matching '$SESSION_PREFIX'${NC}"
    exit 1
fi

if [ $MATCH_COUNT -gt 1 ]; then
    echo -e "${RED}Error: Multiple sessions match '$SESSION_PREFIX'. Please be more specific.${NC}"
    exit 1
fi

SESSION_DIR="sessions/$MATCHED_SESSION"

# Validate session structure
if [ ! -f "$SESSION_DIR/session.json" ]; then
    echo -e "${RED}Error: Invalid session - session.json not found${NC}"
    exit 1
fi

# Get current active session to check if already active
CURRENT_SESSION=""
if [ -f "active.json" ]; then
    if command -v python3 &> /dev/null; then
        CURRENT_SESSION=$(python3 -c "import json; print(json.load(open('active.json'))['id'])" 2>/dev/null || echo "")
    elif command -v jq &> /dev/null; then
        CURRENT_SESSION=$(jq -r '.id' active.json 2>/dev/null || echo "")
    fi
fi

if [ "$MATCHED_SESSION" = "$CURRENT_SESSION" ]; then
    echo -e "${YELLOW}Already on session ${MATCHED_SESSION:0:8}${NC}"
    exit 0
fi

# Update active.json
cat > "active.json" <<EOF
{
  "id": "$MATCHED_SESSION",
  "path": "sessions/$MATCHED_SESSION",
  "schema": 1
}
EOF

echo -e "${GREEN}✓ Switched to session: ${MATCHED_SESSION:0:8}${NC}"

# Display session info
if command -v python3 &> /dev/null; then
    python3 <<EOF
import json
with open('$SESSION_DIR/session.json') as f:
    data = json.load(f)
    print(f"  Name: {data.get('name', 'unknown')}")
    print(f"  Full ID: $MATCHED_SESSION")
EOF
elif command -v jq &> /dev/null; then
    echo "  Name: $(jq -r '.name // "unknown"' "$SESSION_DIR/session.json")"
    echo "  Full ID: $MATCHED_SESSION"
fi

echo ""
echo -e "${BLUE}Note:${NC} Restart your snek LSP for changes to take effect"
