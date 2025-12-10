---
description: Show outline/structure of a file
allowed-tools: Bash(*), Read(*)
---

# File Outline

Display the structure of a source file (functions, classes, structs, etc.).

## Usage

User provides the file path to analyze.

## Steps

1. Run the outline script:

```bash
.snek/scripts/get-file-outline.sh "/path/to/file.rs"
```

2. Display results showing:
   - Line numbers
   - Function/class/struct definitions
   - Type of each item

The script supports: Rust, Python, JavaScript/TypeScript, Go, Java, C/C++
