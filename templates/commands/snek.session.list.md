---
description: List all snek sessions
allowed-tools: Bash(*)
---

# List Sessions

List all available snek sessions with their metadata.

## Steps

1. Run the list-sessions script:

```bash
.snek/scripts/list-sessions.sh
```

2. Display the results in a readable format:
   - Session ID (first 8 chars)
   - Session name
   - Whether it's active
   - Number of context files
   - Number of code snippets
   - Last updated time
