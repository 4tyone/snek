---
description: Generate module/file documentation
allowed-tools: Read(*), Bash(*)
---

# Document File

Generate documentation for an entire file/module.

## Usage

User provides the file path.

## Steps

1. Get file outline:

```bash
.snek/scripts/get-file-outline.sh "/path/to/file"
```

2. Get file imports:

```bash
.snek/scripts/get-imports.sh "/path/to/file"
```

3. Read the file to understand its purpose.

4. Generate module documentation including:
   - Module purpose and overview
   - Main types/structs defined
   - Public functions and their purposes
   - Dependencies and what they're used for
   - Usage examples

5. Format for the language:
   - Rust: `//!` module docs
   - Python: module docstring
   - JavaScript: JSDoc module header

6. Present documentation for user to apply.
