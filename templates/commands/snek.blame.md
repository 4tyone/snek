---
description: Show git blame for a file or line range
allowed-tools: Bash(*)
---

# Git Blame

Show who last modified each line of a file.

## Usage

User provides:
- File path
- Optional: line range (e.g., "10-25")

## Steps

1. Run git blame:

```bash
# Entire file
git blame "/path/to/file"

# Specific lines
git blame -L START,END "/path/to/file"
```

2. Display results showing:
   - Commit hash
   - Author
   - Date
   - Line content
