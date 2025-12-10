---
description: Show dependencies/imports of a file
allowed-tools: Bash(*), Read(*)
---

# File Dependencies

Display the import/use statements from a source file.

## Usage

User provides the file path to analyze.

## Steps

1. Run the imports script:

```bash
.snek/scripts/get-imports.sh "/path/to/file.rs"
```

2. Display results showing:
   - Line numbers
   - Import statements
   - Dependencies used

The script supports: Rust, Python, JavaScript/TypeScript, Go, Java, C/C++
