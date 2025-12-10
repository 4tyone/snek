#!/usr/bin/env bash
#
# get-file-outline.sh - Get outline/structure of a source file
#
# Usage:
#   ./get-file-outline.sh <file_path>
#
# Output: JSON with functions, classes, structs found in file

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

# Use language-specific patterns
case "$EXT" in
    rs)
        # Rust: fn, struct, enum, impl, trait, mod
        grep -n -E "^[[:space:]]*(pub[[:space:]]+)?(fn|struct|enum|impl|trait|mod)[[:space:]]+" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{print "{\"line\": " $1 ", \"text\": \"" $2 "\"}"}' | \
        sed 's/"/\\"/g; s/\\"{/"{/g; s/}\\"/}"/g' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    py)
        # Python: def, class, async def
        grep -n -E "^[[:space:]]*(async[[:space:]]+)?(def|class)[[:space:]]+" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{print "{\"line\": " $1 ", \"text\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    js|ts|tsx|jsx)
        # JS/TS: function, class, const/let/var with arrow functions
        grep -n -E "^[[:space:]]*(export[[:space:]]+)?(async[[:space:]]+)?(function|class)[[:space:]]+|^[[:space:]]*(export[[:space:]]+)?(const|let|var)[[:space:]]+[a-zA-Z_][a-zA-Z0-9_]*[[:space:]]*=" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{print "{\"line\": " $1 ", \"text\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    go)
        # Go: func, type, interface
        grep -n -E "^(func|type)[[:space:]]+" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{print "{\"line\": " $1 ", \"text\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    java)
        # Java: class, interface, public/private methods
        grep -n -E "^[[:space:]]*(public|private|protected)?[[:space:]]*(static[[:space:]]+)?(class|interface|void|[A-Z][a-zA-Z]*)[[:space:]]+" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{print "{\"line\": " $1 ", \"text\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    c|cpp|cc|h|hpp)
        # C/C++: functions and classes
        grep -n -E "^[a-zA-Z_][a-zA-Z0-9_*[:space:]]+[a-zA-Z_][a-zA-Z0-9_]*[[:space:]]*\(|^[[:space:]]*(class|struct)[[:space:]]+" "$FILE_PATH" 2>/dev/null | \
        awk -F: '{print "{\"line\": " $1 ", \"text\": \"" $2 "\"}"}' | \
        awk 'BEGIN{print "["} NR>1{print ","} {print} END{print "]"}'
        ;;
    *)
        echo "{\"error\": \"Unsupported file type: $EXT\"}"
        exit 1
        ;;
esac
