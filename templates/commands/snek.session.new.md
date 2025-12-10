---
description: Create a new snek session
allowed-tools: Bash(*)
---

# Create New Session

Create a new snek session for organizing context around a specific task or feature.

## Usage

The user should provide a session name. If not provided, use "default".

## Steps

1. Run the new-session script:

```bash
.snek/scripts/new-session.sh "SESSION_NAME"
```

2. Report the created session ID and path to the user.

3. Suggest next steps:
   - Add markdown context files to describe the task
   - Add code snippet references for relevant files
