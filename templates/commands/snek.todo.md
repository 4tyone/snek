---
description: Find and list TODOs in codebase
allowed-tools: Bash(*), Grep(*)
---

# Find TODOs

Search for TODO, FIXME, HACK, XXX comments in the codebase.

## Usage

User can optionally specify:
- File pattern to search
- Specific keyword (TODO, FIXME, etc.)

## Steps

1. Search for comment markers:

```bash
grep -rn -E "(TODO|FIXME|HACK|XXX|NOTE):" --include="*.rs" --include="*.py" --include="*.js" --include="*.ts" --include="*.go" .
```

2. Display results grouped by type:
   - TODOs
   - FIXMEs
   - HACKs
   - etc.

3. Show count and file distribution.
