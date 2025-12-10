#!/usr/bin/env bash
#
# get-recent-commits.sh - Get recent git commits
#
# Usage:
#   ./get-recent-commits.sh [count]
#
# Default count: 10

set -euo pipefail

COUNT="${1:-10}"

if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "{\"error\": \"Not a git repository\"}"
    exit 1
fi

git log -n "$COUNT" --pretty=format:'{"hash": "%H", "short_hash": "%h", "author": "%an", "date": "%ci", "subject": "%s"},' | \
sed '$ s/,$//' | \
awk 'BEGIN{print "["} {print} END{print "]"}'
