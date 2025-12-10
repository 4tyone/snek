---
description: Generate documentation for a function
allowed-tools: Read(*), Bash(*)
---

# Document Function

Generate documentation for a specific function.

## Usage

User provides:
- File path
- Function name or line number

## Steps

1. Get file outline to locate function:

```bash
.snek/scripts/get-file-outline.sh "/path/to/file"
```

2. Read the function code.

3. Generate documentation including:
   - Brief description of what the function does
   - Parameters with types and descriptions
   - Return value description
   - Example usage (if applicable)
   - Any important notes or caveats

4. Format documentation in the appropriate style for the language:
   - Rust: `///` doc comments
   - Python: docstring with Google/NumPy style
   - JavaScript/TypeScript: JSDoc
   - Go: godoc style

5. Present the documentation for user to apply.
