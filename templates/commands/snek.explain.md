---
description: Explain code at a specific location
allowed-tools: Read(*), Bash(*)
---

# Explain Code

Explain what a piece of code does.

## Usage

User provides:
- File path
- Line number or line range (e.g., "10-25")
- Or: function/struct name to find and explain

## Steps

1. If line range provided, read that section:
   - Read the file
   - Extract the specified lines

2. If function/struct name provided:
   - Use outline script to find it:
   ```bash
   .snek/scripts/get-file-outline.sh "/path/to/file"
   ```
   - Read the relevant section

3. Analyze and explain:
   - What the code does
   - Key logic and data flow
   - Important patterns or techniques used
   - Any potential issues or considerations
