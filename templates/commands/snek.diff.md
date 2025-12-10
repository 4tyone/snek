---
description: Show and explain git diff
allowed-tools: Bash(*)
---

# Git Diff

Show git diff and optionally explain the changes.

## Usage

User can specify:
- `--staged` to show staged changes
- `--file PATH` to show diff for specific file
- No args shows unstaged changes

## Steps

1. Get the diff:

```bash
# Unstaged changes
.snek/scripts/get-git-diff.sh

# Staged changes
.snek/scripts/get-git-diff.sh --staged

# Specific file
.snek/scripts/get-git-diff.sh --file "/path/to/file"
```

2. Display the diff output.

3. If user asks for explanation, summarize:
   - What files changed
   - Nature of changes (additions, removals, modifications)
   - Key code changes and their purpose
