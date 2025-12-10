---
description: Find callers of a function
allowed-tools: Bash(*), Grep(*), Read(*)
---

# Find Callers

Find all places where a function is called in the codebase.

## Usage

User provides:
- Function name
- Optional: file pattern to limit search

## Steps

1. Search for function calls:

```bash
# Search for function call patterns
grep -rn "FUNCTION_NAME(" --include="*.EXTENSION" .
```

2. Filter out the function definition itself.

3. Display results with:
   - Caller file and line
   - Context showing the call

4. Summarize how many places call this function.
