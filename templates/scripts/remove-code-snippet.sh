#!/usr/bin/env bash
#
# remove-code-snippet.sh - Remove a code snippet by URI or index
#
# Usage:
#   ./remove-code-snippet.sh <uri_or_index>
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

if [ $# -eq 0 ]; then
    echo "{\"error\": \"URI or index required\"}"
    exit 1
fi

TARGET="$1"

# Get active session path
if [ ! -f "active.json" ]; then
    echo "{\"error\": \"No active session\"}"
    exit 1
fi

SESSION_PATH=$(python3 -c "import json; print(json.load(open('active.json')).get('path', ''))" 2>/dev/null)
SNIPPETS_FILE="$SESSION_PATH/code_snippets.json"

if [ ! -f "$SNIPPETS_FILE" ]; then
    echo "{\"error\": \"No snippets file\"}"
    exit 1
fi

python3 <<EOF
import json

with open('$SNIPPETS_FILE', 'r') as f:
    data = json.load(f)

target = '$TARGET'
removed = None

# Try as index first
try:
    idx = int(target)
    if 0 <= idx < len(data['snippets']):
        removed = data['snippets'].pop(idx)
except ValueError:
    # Try as URI
    for i, snippet in enumerate(data['snippets']):
        if target in snippet.get('uri', ''):
            removed = data['snippets'].pop(i)
            break

if removed:
    with open('$SNIPPETS_FILE', 'w') as f:
        json.dump(data, f, indent=2)
    print(json.dumps({'success': True, 'removed': removed}))
else:
    print(json.dumps({'error': 'Snippet not found'}))
EOF
