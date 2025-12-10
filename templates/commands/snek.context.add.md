---
description: Add context to active session
allowed-tools: Bash(*), Read(*), Write(*)
---

# Add Context

Add markdown context or code snippet references to the active session.

## Usage

User should specify what to add:
- For markdown: provide filename and content (or let Claude write it)
- For code snippet: provide file path and optionally line range

## Steps

### Adding Markdown Context

1. Create the context file:

```bash
.snek/scripts/add-context-file.sh "filename.md" "CONTENT_HERE"
```

Or for longer content, write directly:

```bash
# First get session path
.snek/scripts/get-active-session.sh
# Then write to .snek/SESSION_PATH/context/filename.md
```

### Adding Code Snippet

1. Add the code reference:

```bash
.snek/scripts/add-code-snippet.sh "/absolute/path/to/file.rs" START_LINE END_LINE "Description"
```

If no line range specified, entire file is included.

2. Confirm what was added and show updated context list.
