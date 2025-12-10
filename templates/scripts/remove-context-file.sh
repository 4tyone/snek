#!/usr/bin/env bash
#
# remove-context-file.sh - Remove a markdown context file from active session
#
# Usage:
#   ./remove-context-file.sh <filename.md>
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

# Get active session path
if [ ! -f "active.json" ]; then
    echo "{\"error\": \"No active session\"}"
    exit 1
fi

SESSION_PATH=$(python3 -c "import json; print(json.load(open('active.json')).get('path', ''))" 2>/dev/null)

TARGET_FILE="$SESSION_PATH/context/$FILENAME"

if [ ! -f "$TARGET_FILE" ]; then
    echo "{\"error\": \"File not found: $FILENAME\"}"
    exit 1
fi

rm "$TARGET_FILE"
echo "{\"success\": true, \"removed\": \"$FILENAME\"}"
