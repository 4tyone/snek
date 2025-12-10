#!/usr/bin/env bash
#
# list-context.sh - List all context files and code snippets in active session
#
# Output: JSON object with markdown files and code snippets

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

if [ ! -f "active.json" ]; then
    echo "{\"error\": \"No active session\"}"
    exit 1
fi

if command -v python3 &> /dev/null; then
    python3 <<EOF
import json
import os

with open('active.json') as f:
    active = json.load(f)

session_path = active.get('path', '')
result = {
    'session_id': active.get('id', ''),
    'markdown_files': [],
    'code_snippets': []
}

# List markdown files
context_dir = os.path.join(session_path, 'context')
if os.path.isdir(context_dir):
    for f in sorted(os.listdir(context_dir)):
        if f.endswith('.md'):
            filepath = os.path.join(context_dir, f)
            stat = os.stat(filepath)
            result['markdown_files'].append({
                'name': f,
                'path': filepath,
                'size_bytes': stat.st_size
            })

# List code snippets
snippets_file = os.path.join(session_path, 'code_snippets.json')
if os.path.exists(snippets_file):
    with open(snippets_file) as f:
        snippets_data = json.load(f)
        for snippet in snippets_data.get('snippets', []):
            result['code_snippets'].append({
                'uri': snippet.get('uri', ''),
                'lines': f"{snippet.get('start_line', 0)}-{snippet.get('end_line', 0)}",
                'language': snippet.get('language_id', ''),
                'description': snippet.get('description', '')
            })

print(json.dumps(result, indent=2))
EOF
else
    echo "{\"error\": \"python3 required\"}"
fi
