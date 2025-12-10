---
description: Draft a commit message for current changes
allowed-tools: Bash(*), Read(*)
---

# Draft Commit Message

Analyze current changes and draft a commit message.

## Steps

1. Get git status and diff:

```bash
.snek/scripts/get-git-status.sh
.snek/scripts/get-git-diff.sh
.snek/scripts/get-git-diff.sh --staged
```

2. Analyze the changes:
   - What files were modified
   - Nature of changes
   - Purpose of the changes

3. Draft a commit message following conventional commit format:
   - Type: feat, fix, refactor, docs, test, chore
   - Scope (optional): component or area affected
   - Subject: imperative, lowercase, no period
   - Body (if needed): explain what and why

Example format:
```
feat(session): add hot-reload for session switching

- Watch active.json for changes
- Automatically reload context on session switch
- No window reload required
```

4. Present the draft to user for approval or modification.
