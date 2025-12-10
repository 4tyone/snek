#!/usr/bin/env bash
#
# add-context-file.sh - Add a markdown context file to active session
#
# Usage:
#   ./add-context-file.sh <filename.md> <content>
#   echo "content" | ./add-context-file.sh <filename.md>
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

if [ $# -eq 0 ]; then
    echo "{\"error\": \"Filename required\"}"
    exit 1
fi

FILENAME="$1"

# Ensure .md extension
if [[ "$FILENAME" != *.md ]]; then
    FILENAME="${FILENAME}.md"
fi

# Get active session path
if [ ! -f "active.json" ]; then
    echo "{\"error\": \"No active session\"}"
    exit 1
fi

SESSION_PATH=$(python3 -c "import json; print(json.load(open('active.json')).get('path', ''))" 2>/dev/null)

if [ -z "$SESSION_PATH" ]; then
    echo "{\"error\": \"Could not determine session path\"}"
    exit 1
fi

CONTEXT_DIR="$SESSION_PATH/context"
mkdir -p "$CONTEXT_DIR"

TARGET_FILE="$CONTEXT_DIR/$FILENAME"

# Get content from argument or stdin
if [ $# -ge 2 ]; then
    echo "$2" > "$TARGET_FILE"
else
    cat > "$TARGET_FILE"
fi

echo "{\"success\": true, \"path\": \"$TARGET_FILE\"}"
