#!/usr/bin/env bash
#
# get-active-session.sh - Get the currently active session info
#
# Output: JSON object with session details

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

if [ ! -f "active.json" ]; then
    echo "{\"error\": \"No active session found\"}"
    exit 1
fi

if command -v python3 &> /dev/null; then
    python3 <<EOF
import json
import os

with open('active.json') as f:
    active = json.load(f)

session_id = active.get('id', '')
session_path = active.get('path', '')

result = {
    'id': session_id,
    'path': session_path
}

session_file = os.path.join(session_path, 'session.json')
if os.path.exists(session_file):
    with open(session_file) as f:
        session_data = json.load(f)
        result['name'] = session_data.get('name', 'unknown')
        result['max_tokens'] = session_data.get('limits', {}).get('max_tokens', 1600)
        result['updated_at'] = session_data.get('updated_at', '')

# Count context files
context_dir = os.path.join(session_path, 'context')
if os.path.isdir(context_dir):
    result['context_files'] = [f for f in os.listdir(context_dir) if f.endswith('.md')]
else:
    result['context_files'] = []

# Get code snippets
snippets_file = os.path.join(session_path, 'code_snippets.json')
if os.path.exists(snippets_file):
    with open(snippets_file) as f:
        snippets_data = json.load(f)
        result['code_snippets'] = len(snippets_data.get('snippets', []))
else:
    result['code_snippets'] = 0

print(json.dumps(result, indent=2))
EOF
else
    cat active.json
fi
