---
description: Find references to a symbol in codebase
allowed-tools: Bash(*), Grep(*)
---

# Find References

Search for all references to a symbol (function, variable, type) in the codebase.

## Usage

User provides:
- Symbol name to search for
- Optional: file pattern to limit search (e.g., "*.rs", "src/**/*.ts")

## Steps

1. Search for the symbol:

```bash
# Basic search
grep -rn "SYMBOL_NAME" --include="*.EXTENSION" .

# For more precise function references in Rust
grep -rn "SYMBOL_NAME(" --include="*.rs" .

# For type references
grep -rn ": SYMBOL_NAME" --include="*.rs" .
```

2. Display results with:
   - File path
   - Line number
   - Context line

3. Summarize total occurrences and files affected.
