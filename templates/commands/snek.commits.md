---
description: Show recent git commits
allowed-tools: Bash(*)
---

# Recent Commits

Show recent git commit history.

## Usage

User can specify number of commits (default: 10).

## Steps

1. Get recent commits:

```bash
.snek/scripts/get-recent-commits.sh COUNT
```

2. Display in readable format:
   - Short hash
   - Author
   - Date
   - Commit message
