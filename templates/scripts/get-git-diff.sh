#!/usr/bin/env bash
#
# get-git-diff.sh - Get git diff information
#
# Usage:
#   ./get-git-diff.sh [--staged] [--file <path>]
#
# Output: Git diff output

set -euo pipefail

STAGED=false
FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --staged) STAGED=true; shift ;;
        --file) FILE="$2"; shift 2 ;;
        *) shift ;;
    esac
done

if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "{\"error\": \"Not a git repository\"}"
    exit 1
fi

if [ "$STAGED" = true ]; then
    if [ -n "$FILE" ]; then
        git diff --cached -- "$FILE"
    else
        git diff --cached
    fi
else
    if [ -n "$FILE" ]; then
        git diff -- "$FILE"
    else
        git diff
    fi
fi
