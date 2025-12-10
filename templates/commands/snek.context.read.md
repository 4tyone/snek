---
description: Read a specific context file
allowed-tools: Bash(*), Read(*)
---

# Read Context File

Read and display the contents of a specific markdown context file.

## Usage

User provides the filename of the context file to read.

## Steps

1. Get active session path:

```bash
.snek/scripts/get-active-session.sh
```

2. Read the specified file from the context directory.

3. Display the file contents to the user.

If filename not provided, list available files first:

```bash
.snek/scripts/list-context.sh
```
