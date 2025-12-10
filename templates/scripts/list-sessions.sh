#!/usr/bin/env bash
#
# list-sessions.sh - List all snek sessions with their metadata
#
# Output format: JSON array of sessions
# [
#   {
#     "id": "uuid",
#     "name": "session name",
#     "path": "sessions/uuid",
#     "active": true/false,
#     "context_files": 3,
#     "code_snippets": 5,
#     "updated_at": "timestamp"
#   }
# ]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

# Get active session ID
ACTIVE_ID=""
if [ -f "active.json" ]; then
    if command -v python3 &> /dev/null; then
        ACTIVE_ID=$(python3 -c "import json; print(json.load(open('active.json')).get('id', ''))" 2>/dev/null || echo "")
    elif command -v jq &> /dev/null; then
        ACTIVE_ID=$(jq -r '.id // ""' active.json 2>/dev/null || echo "")
    fi
fi

echo "["

first=true
for session_dir in sessions/*/; do
    if [ -d "$session_dir" ]; then
        session_id=$(basename "$session_dir")

        if [ ! -f "$session_dir/session.json" ]; then
            continue
        fi

        # Get session metadata
        if command -v python3 &> /dev/null; then
            session_data=$(python3 <<EOF
import json
import os

with open('$session_dir/session.json') as f:
    data = json.load(f)

name = data.get('name', 'unknown')
updated_at = data.get('updated_at', '')

# Count context files
context_dir = '$session_dir/context'
context_count = len([f for f in os.listdir(context_dir) if f.endswith('.md')]) if os.path.isdir(context_dir) else 0

# Count code snippets
snippets_file = '$session_dir/code_snippets.json'
snippets_count = 0
if os.path.exists(snippets_file):
    with open(snippets_file) as f:
        snippets_data = json.load(f)
        snippets_count = len(snippets_data.get('snippets', []))

is_active = '$session_id' == '$ACTIVE_ID'

print(json.dumps({
    'id': '$session_id',
    'name': name,
    'path': 'sessions/$session_id',
    'active': is_active,
    'context_files': context_count,
    'code_snippets': snippets_count,
    'updated_at': updated_at
}))
EOF
)
        else
            # Fallback without python
            session_data="{\"id\": \"$session_id\", \"name\": \"unknown\", \"path\": \"sessions/$session_id\", \"active\": false}"
        fi

        if [ "$first" = true ]; then
            first=false
        else
            echo ","
        fi
        echo "  $session_data"
    fi
done

echo ""
echo "]"
