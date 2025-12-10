---
description: Analyze code complexity
allowed-tools: Read(*), Bash(*)
---

# Analyze Complexity

Analyze the complexity of a function or file.

## Usage

User provides:
- File path
- Optional: specific function name

## Steps

1. Get file outline:

```bash
.snek/scripts/get-file-outline.sh "/path/to/file"
```

2. Read and analyze the code for:
   - Cyclomatic complexity (branches, loops)
   - Nesting depth
   - Function length
   - Number of parameters
   - Dependencies

3. Report findings:
   - Complexity score/rating
   - Specific concerns (deeply nested, too many params, etc.)
   - Suggestions for simplification

4. If issues found, suggest refactoring approaches.
