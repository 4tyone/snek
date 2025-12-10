#!/usr/bin/env bash
#
# get-imports.sh - Get import/use statements from a source file
#
# Usage:
#   ./get-imports.sh <file_path>
#
# Output: JSON array of import statements

set -euo pipefail

if [ $# -eq 0 ]; then
    echo "{\"error\": \"File path required\"}"
    exit 1
fi

FILE_PATH="$1"

if [ ! -f "$FILE_PATH" ]; then
    echo "{\"error\": \"File not found\"}"
    exit 1
fi

EXT="${FILE_PATH##*.}"

case "$EXT" in
    rs)
        # Rust: use statements
        grep -n "^[[:space:]]*use[[:space:]]" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{gsub(/^[[:space:]]+/, "", $2); print "{\"line\": " $1 ", \"import\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    py)
        # Python: import and from ... import
        grep -n -E "^[[:space:]]*(import|from)[[:space:]]" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{gsub(/^[[:space:]]+/, "", $2); print "{\"line\": " $1 ", \"import\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    js|ts|tsx|jsx)
        # JS/TS: import statements
        grep -n -E "^[[:space:]]*(import|export[[:space:]]+\{.*\}[[:space:]]+from)" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{gsub(/^[[:space:]]+/, "", $2); print "{\"line\": " $1 ", \"import\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    go)
        # Go: import statements
        grep -n -E "^[[:space:]]*(import[[:space:]]|\")" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{gsub(/^[[:space:]]+/, "", $2); print "{\"line\": " $1 ", \"import\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    java)
        # Java: import statements
        grep -n "^[[:space:]]*import[[:space:]]" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{gsub(/^[[:space:]]+/, "", $2); print "{\"line\": " $1 ", \"import\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    c|cpp|cc|h|hpp)
        # C/C++: #include statements
        grep -n "^[[:space:]]*#include" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{gsub(/^[[:space:]]+/, "", $2); print "{\"line\": " $1 ", \"import\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    *)
        echo "[]"
        ;;
esac
