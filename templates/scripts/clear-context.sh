#!/usr/bin/env bash
#
# clear-context.sh - Clear all context files and/or code snippets
#
# Usage:
#   ./clear-context.sh [--markdown] [--snippets] [--all]
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

CLEAR_MD=false
CLEAR_SNIPPETS=false

# Parse arguments
for arg in "$@"; do
    case $arg in
        --markdown) CLEAR_MD=true ;;
        --snippets) CLEAR_SNIPPETS=true ;;
        --all) CLEAR_MD=true; CLEAR_SNIPPETS=true ;;
    esac
done

# Default to all if no flags
if [ "$CLEAR_MD" = false ] && [ "$CLEAR_SNIPPETS" = false ]; then
    CLEAR_MD=true
    CLEAR_SNIPPETS=true
fi

# Get active session path
if [ ! -f "active.json" ]; then
    echo "{\"error\": \"No active session\"}"
    exit 1
fi

SESSION_PATH=$(python3 -c "import json; print(json.load(open('active.json')).get('path', ''))" 2>/dev/null)

RESULT="{\"cleared\": {"
FIRST=true

if [ "$CLEAR_MD" = true ]; then
    CONTEXT_DIR="$SESSION_PATH/context"
    if [ -d "$CONTEXT_DIR" ]; then
        COUNT=$(find "$CONTEXT_DIR" -name "*.md" | wc -l | tr -d ' ')
        rm -f "$CONTEXT_DIR"/*.md 2>/dev/null || true
        RESULT="${RESULT}\"markdown_files\": $COUNT"
        FIRST=false
    fi
fi

if [ "$CLEAR_SNIPPETS" = true ]; then
    SNIPPETS_FILE="$SESSION_PATH/code_snippets.json"
    if [ -f "$SNIPPETS_FILE" ]; then
        COUNT=$(python3 -c "import json; print(len(json.load(open('$SNIPPETS_FILE')).get('snippets', [])))" 2>/dev/null || echo "0")
        echo '{"schema": 1, "snippets": []}' > "$SNIPPETS_FILE"
        if [ "$FIRST" = false ]; then
            RESULT="${RESULT}, "
        fi
        RESULT="${RESULT}\"code_snippets\": $COUNT"
    fi
fi

RESULT="${RESULT}}}"
echo "$RESULT"
