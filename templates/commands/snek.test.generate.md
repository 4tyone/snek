---
description: Generate tests for a function or module
allowed-tools: Read(*), Bash(*), Write(*)
---

# Generate Tests

Generate unit tests for a function or module.

## Usage

User provides:
- File path
- Function name (optional - if not provided, generate tests for all public functions)

## Steps

1. Get file outline:

```bash
.snek/scripts/get-file-outline.sh "/path/to/file"
```

2. Read the function(s) to understand:
   - Input parameters and types
   - Return type
   - Edge cases
   - Error conditions

3. Generate tests covering:
   - Happy path (normal operation)
   - Edge cases (empty input, boundary values)
   - Error cases (invalid input, error conditions)

4. Format tests for the language:
   - Rust: `#[test]` functions
   - Python: pytest functions
   - JavaScript/TypeScript: Jest/Mocha tests
   - Go: `func TestXxx` functions

5. Present tests to user. If approved, write to appropriate test file location.
