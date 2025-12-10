#!/usr/bin/env bash
#
# add-code-snippet.sh - Add a code snippet reference to active session
#
# Usage:
#   ./add-code-snippet.sh <file_path> [start_line] [end_line] [description]
#
# If start_line/end_line not provided, includes entire file

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

if [ $# -eq 0 ]; then
    echo "{\"error\": \"File path required\"}"
    exit 1
fi

FILE_PATH="$1"
START_LINE="${2:-0}"
END_LINE="${3:-}"
DESCRIPTION="${4:-}"

# Convert to absolute path
if [[ "$FILE_PATH" != /* ]]; then
    FILE_PATH="$(cd "$(dirname "$FILE_PATH")" && pwd)/$(basename "$FILE_PATH")"
fi

if [ ! -f "$FILE_PATH" ]; then
    echo "{\"error\": \"File not found: $FILE_PATH\"}"
    exit 1
fi

# If end_line not provided, use total line count
if [ -z "$END_LINE" ]; then
    END_LINE=$(wc -l < "$FILE_PATH" | tr -d ' ')
fi

# Detect language from extension
EXT="${FILE_PATH##*.}"
case "$EXT" in
    rs) LANG="rust" ;;
    py) LANG="python" ;;
    js) LANG="javascript" ;;
    ts) LANG="typescript" ;;
    go) LANG="go" ;;
    c) LANG="c" ;;
    cpp|cc|cxx) LANG="cpp" ;;
    java) LANG="java" ;;
    lua) LANG="lua" ;;
    *) LANG="$EXT" ;;
esac

# Get active session path
if [ ! -f "active.json" ]; then
    echo "{\"error\": \"No active session\"}"
    exit 1
fi

SESSION_PATH=$(python3 -c "import json; print(json.load(open('active.json')).get('path', ''))" 2>/dev/null)
SNIPPETS_FILE="$SESSION_PATH/code_snippets.json"

# Create snippets file if it doesn't exist
if [ ! -f "$SNIPPETS_FILE" ]; then
    echo '{"schema": 1, "snippets": []}' > "$SNIPPETS_FILE"
fi

# Add snippet using python
python3 <<EOF
import json
from datetime import datetime, timezone

with open('$SNIPPETS_FILE', 'r') as f:
    data = json.load(f)

new_snippet = {
    'uri': 'file://$FILE_PATH',
    'start_line': $START_LINE,
    'end_line': $END_LINE,
    'language_id': '$LANG',
    'description': '$DESCRIPTION' if '$DESCRIPTION' else None,
    'last_modified': datetime.now(timezone.utc).isoformat()
}

# Remove None values
new_snippet = {k: v for k, v in new_snippet.items() if v is not None}

data['snippets'].append(new_snippet)

with open('$SNIPPETS_FILE', 'w') as f:
    json.dump(data, f, indent=2)

print(json.dumps({'success': True, 'snippet': new_snippet}))
EOF
