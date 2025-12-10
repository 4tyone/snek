---
description: Clone an existing session
allowed-tools: Bash(*)
---

# Clone Session

Create a copy of an existing session with all its context and code snippets.

## Usage

User should provide:
- Source session ID prefix
- New session name (optional)

## Steps

1. If no source session provided, list available sessions:

```bash
.snek/scripts/list-sessions.sh
```

2. Clone the session:

```bash
.snek/scripts/clone-session.sh "SOURCE_SESSION_PREFIX" "NEW_NAME"
```

3. Report the new session ID and confirm it was created.
