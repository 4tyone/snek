---
description: Extract code into a new function
allowed-tools: Read(*), Edit(*), Bash(*)
---

# Extract Function

Extract selected code into a new function.

## Usage

User provides:
- File path
- Start line and end line of code to extract
- New function name

## Steps

1. Read the specified code section.

2. Analyze:
   - Variables used from outer scope (become parameters)
   - Variables modified that are used later (become return values)
   - Dependencies and imports needed

3. Generate:
   - New function with appropriate signature
   - Call site replacing the original code

4. Present the refactoring plan:
   - Show the new function
   - Show the replacement call
   - List any considerations

5. If approved, apply the changes using Edit tool.
