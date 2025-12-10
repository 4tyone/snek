---
description: Rename a symbol across the codebase
allowed-tools: Bash(*), Grep(*), Read(*), Edit(*)
---

# Rename Symbol

Rename a function, variable, type, or other symbol across the codebase.

## Usage

User provides:
- Old name
- New name
- Optional: scope (single file or entire project)

## Steps

1. Find all occurrences:

```bash
grep -rn "OLD_NAME" --include="*.EXTENSION" .
```

2. Categorize occurrences:
   - Definition
   - References/usages
   - String literals (may need different handling)
   - Comments/docs

3. Show preview of all changes.

4. If user approves, apply changes file by file using Edit tool.

5. Verify no syntax errors after rename.
