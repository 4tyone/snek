---
description: Switch to a different snek session
allowed-tools: Bash(*)
---

# Switch Session

Switch the active snek session.

## Usage

The user should provide a session ID prefix (first 8 characters is enough).

If no ID provided, first list available sessions so the user can choose.

## Steps

1. If no session ID provided, list available sessions:

```bash
.snek/scripts/list-sessions.sh
```

2. Once user provides session ID, switch to it:

```bash
.snek/scripts/switch-session.sh "SESSION_ID_PREFIX"
```

3. Confirm the switch was successful and show the new session details.
