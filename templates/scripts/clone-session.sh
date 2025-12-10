#!/usr/bin/env bash
#
# clone-session.sh - Clone an existing session to a new one
#
# Usage:
#   ./clone-session.sh <source_session_prefix> [new_name]
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNEK_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SNEK_ROOT" || exit 1

if [ $# -eq 0 ]; then
    echo "Error: Source session ID prefix required"
    echo "Usage: $0 <source_session_prefix> [new_name]"
    exit 1
fi

SOURCE_PREFIX="$1"
NEW_NAME="${2:-cloned}"

# Find source session
SOURCE_SESSION=""
for session_dir in sessions/*/; do
    if [ -d "$session_dir" ]; then
        session_id=$(basename "$session_dir")
        if [[ "$session_id" == "$SOURCE_PREFIX"* ]]; then
            SOURCE_SESSION="$session_id"
            break
        fi
    fi
done

if [ -z "$SOURCE_SESSION" ]; then
    echo "Error: No session found matching '$SOURCE_PREFIX'"
    exit 1
fi

SOURCE_DIR="sessions/$SOURCE_SESSION"

# Generate new UUID
if command -v uuidgen &> /dev/null; then
    NEW_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
elif command -v python3 &> /dev/null; then
    NEW_ID=$(python3 -c "import uuid; print(str(uuid.uuid4()))")
else
    echo "Error: No UUID generator found"
    exit 1
fi

NEW_DIR="sessions/$NEW_ID"

# Copy session directory
cp -r "$SOURCE_DIR" "$NEW_DIR"

# Update session.json with new ID and name
if command -v python3 &> /dev/null; then
    python3 <<EOF
import json
from datetime import datetime, timezone

with open('$NEW_DIR/session.json', 'r') as f:
    data = json.load(f)

data['id'] = '$NEW_ID'
data['name'] = '$NEW_NAME'
data['updated_at'] = datetime.now(timezone.utc).isoformat()

with open('$NEW_DIR/session.json', 'w') as f:
    json.dump(data, f, indent=2)
EOF
fi

echo "{\"id\": \"$NEW_ID\", \"name\": \"$NEW_NAME\", \"source\": \"$SOURCE_SESSION\", \"path\": \"sessions/$NEW_ID\"}"
