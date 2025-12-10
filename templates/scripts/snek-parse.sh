#!/bin/bash
# snek-parse.sh - Find and parse all @@snek ... snek@@ blocks in the codebase
#
# Output format: JSON array of blocks
# [
#   {
#     "file": "/absolute/path/to/file.go",
#     "start_line": 10,
#     "end_line": 15,
#     "content": "raw text between markers"
#   },
#   ...
# ]

set -e

# Get the project root (where .claude directory is)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# File extensions to ignore (documentation, non-source files)
IGNORE_EXTENSIONS=(
    "*.png"
    "*.jpg"
    "*.jpeg"
    "*.gif"
    "*.ico"
    "*.pdf"
    "*.lock"
    "*.sum"
    "*.mod"
    "*.md"
)

# Directories to ignore (common across all languages)
IGNORE_DIRS=(
    # Version control
    ".git"
    ".svn"
    ".hg"

    # IDE/Editor
    ".idea"
    ".vscode"
    ".claude"
    ".cursor"

    # JavaScript/TypeScript/Node
    "node_modules"
    ".next"
    ".nuxt"
    ".output"
    "bower_components"

    # Python
    "__pycache__"
    ".venv"
    "venv"
    "env"
    ".env"
    ".tox"
    ".nox"
    ".pytest_cache"
    ".mypy_cache"
    ".ruff_cache"
    "*.egg-info"
    ".eggs"
    "site-packages"

    # Go
    "vendor"

    # Rust
    "target"

    # Java/Kotlin/Scala
    ".gradle"
    ".maven"
    "build"
    "out"
    ".settings"

    # .NET/C#
    "bin"
    "obj"
    "packages"
    ".nuget"

    # Ruby
    ".bundle"

    # PHP
    "vendor"

    # Swift/iOS
    ".build"
    "Pods"
    "DerivedData"

    # General build/dist
    "dist"
    "build"
    "_build"
    "release"
    "debug"

    # Test coverage
    "coverage"
    ".coverage"
    "htmlcov"
    ".nyc_output"

    # Documentation
    "docs/_build"
    "_site"
    ".docusaurus"

    # Logs and temp
    "logs"
    "tmp"
    "temp"
    ".tmp"
    ".temp"
    ".cache"

    # OS files
    ".DS_Store"
    "Thumbs.db"
)

# Build find command with all exclusions
# We use eval to properly handle the glob patterns
FIND_CMD="find \"$PROJECT_ROOT\""

# Add directory prunes
FIND_CMD="$FIND_CMD \\("
first_dir=true
for dir in "${IGNORE_DIRS[@]}"; do
    if [ "$first_dir" = true ]; then
        first_dir=false
    else
        FIND_CMD="$FIND_CMD -o"
    fi
    FIND_CMD="$FIND_CMD -name \"$dir\" -type d"
done
FIND_CMD="$FIND_CMD \\) -prune -o -type f"

# Add file extension exclusions
for ext in "${IGNORE_EXTENSIONS[@]}"; do
    FIND_CMD="$FIND_CMD ! -name \"$ext\""
done

FIND_CMD="$FIND_CMD -print"

# Find all source files containing @@snek (excluding ignored dirs and extensions)
FILES=$(eval "$FIND_CMD" 2>/dev/null | xargs grep -l '@@snek' 2>/dev/null || true)

if [ -z "$FILES" ]; then
    echo "[]"
    exit 0
fi

# Start JSON array
echo "["

first=true

for file in $FILES; do
    # Skip binary files
    if file "$file" | grep -q "binary\|executable\|archive\|image\|audio\|video"; then
        continue
    fi

    # Use awk to find and extract snek blocks
    # This handles multi-line blocks and blocks that span across code
    awk -v filename="$file" -v first="$first" '
    BEGIN {
        in_block = 0
        block_content = ""
        start_line = 0
        needs_comma = (first == "false")
    }

    # Match @@snek start (with optional inline content)
    /@@snek/ {
        if (in_block) {
            # Nested block - error case, skip
            next
        }
        in_block = 1
        start_line = NR

        # Check for inline syntax: @@snek: ... snek@@ on same line
        if (match($0, /@@snek:.*snek@@/)) {
            # Extract content between @@snek: and snek@@
            content = $0
            gsub(/.*@@snek:/, "", content)
            gsub(/snek@@.*/, "", content)

            # Escape for JSON
            gsub(/\\/, "\\\\", content)
            gsub(/"/, "\\\"", content)
            gsub(/\t/, "\\t", content)
            gsub(/\r/, "", content)

            if (needs_comma) printf ","
            printf "\n  {\n"
            printf "    \"file\": \"%s\",\n", filename
            printf "    \"start_line\": %d,\n", NR
            printf "    \"end_line\": %d,\n", NR
            printf "    \"content\": \"%s\"\n", content
            printf "  }"
            needs_comma = 1
            in_block = 0
            next
        }

        # Multi-line block - capture everything after @@snek on this line
        content = $0
        gsub(/.*@@snek/, "", content)
        if (content != "") {
            block_content = content "\n"
        }
        next
    }

    # Match snek@@ end
    /snek@@/ {
        if (!in_block) {
            # Orphan closing tag - skip
            next
        }

        # Capture everything before snek@@ on this line
        content = $0
        gsub(/snek@@.*/, "", content)
        if (content != "" && content !~ /^[[:space:]]*$/) {
            block_content = block_content content
        }

        # Remove trailing newline if present
        gsub(/\n$/, "", block_content)

        # Escape for JSON
        gsub(/\\/, "\\\\", block_content)
        gsub(/"/, "\\\"", block_content)
        gsub(/\t/, "\\t", block_content)
        gsub(/\r/, "", block_content)
        gsub(/\n/, "\\n", block_content)

        if (needs_comma) printf ","
        printf "\n  {\n"
        printf "    \"file\": \"%s\",\n", filename
        printf "    \"start_line\": %d,\n", start_line
        printf "    \"end_line\": %d,\n", NR
        printf "    \"content\": \"%s\"\n", block_content
        printf "  }"
        needs_comma = 1

        in_block = 0
        block_content = ""
        start_line = 0
        next
    }

    # Inside a block - accumulate content
    in_block {
        block_content = block_content $0 "\n"
    }

    END {
        if (in_block) {
            # Unclosed block - output with error marker
            gsub(/\\/, "\\\\", block_content)
            gsub(/"/, "\\\"", block_content)
            gsub(/\t/, "\\t", block_content)
            gsub(/\r/, "", block_content)
            gsub(/\n/, "\\n", block_content)

            if (needs_comma) printf ","
            printf "\n  {\n"
            printf "    \"file\": \"%s\",\n", filename
            printf "    \"start_line\": %d,\n", start_line
            printf "    \"end_line\": -1,\n"
            printf "    \"content\": \"%s\",\n", block_content
            printf "    \"error\": \"unclosed block\"\n"
            printf "  }"
        }
    }
    ' "$file"

    first=false
done

# End JSON array
echo ""
echo "]"
