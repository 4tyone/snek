---
description: Show all context in active session
allowed-tools: Bash(*)
---

# Show Context

Display all markdown context files and code snippets in the active session.

## Steps

1. Get context listing:

```bash
.snek/scripts/list-context.sh
```

2. Display in readable format:
   - Markdown files with sizes
   - Code snippets with URIs, line ranges, and descriptions
