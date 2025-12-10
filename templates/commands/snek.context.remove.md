---
description: Remove context from active session
allowed-tools: Bash(*)
---

# Remove Context

Remove markdown context files or code snippet references from the active session.

## Usage

User should specify:
- For markdown: filename to remove
- For code snippet: URI or index number

## Steps

### Removing Markdown

1. List current context to show what's available:

```bash
.snek/scripts/list-context.sh
```

2. Remove the specified file:

```bash
.snek/scripts/remove-context-file.sh "filename.md"
```

### Removing Code Snippet

1. List current snippets:

```bash
.snek/scripts/list-context.sh
```

2. Remove by index or URI:

```bash
.snek/scripts/remove-code-snippet.sh INDEX_OR_URI
```

3. Confirm removal and show updated context.
