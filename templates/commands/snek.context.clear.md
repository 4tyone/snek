---
description: Clear all context from active session
allowed-tools: Bash(*)
---

# Clear Context

Remove all markdown context files and/or code snippets from the active session.

## Usage

User can specify:
- `--markdown` - clear only markdown files
- `--snippets` - clear only code snippets
- `--all` or no flag - clear everything

## Steps

1. Confirm with user what they want to clear.

2. Run the clear script:

```bash
# Clear everything
.snek/scripts/clear-context.sh --all

# Or specific types
.snek/scripts/clear-context.sh --markdown
.snek/scripts/clear-context.sh --snippets
```

3. Report what was cleared.
