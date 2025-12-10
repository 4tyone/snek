# Snek Fill Command

You are processing `@@snek` blocks - regions in source code where the developer has written specifications for code that needs to be generated or modified.

## Step 1: Find all snek blocks

Run the snek parser script to find all `@@snek ... snek@@` blocks in the codebase:

```bash
.claude/scripts/snek-parse.sh
```

This will output JSON with all blocks found. Each block contains:
- `file`: Full path to the file
- `start_line`: Line number where `@@snek` starts
- `end_line`: Line number where `snek@@` ends
- `content`: The raw text between the markers (specification, code, or both)

## Step 2: Create todo list

For EACH block found, create a todo item. This is CRITICAL - every single block must be tracked:

```
TodoWrite with items like:
- "Implement snek block in {file}:{start_line}-{end_line}"
```

## Step 3: Process each block

For each block, in order:

### 3a. Read the file and understand context
- Read the file containing the block
- If the content references other files via `@path/to/file` syntax, read those files too
- Understand the surrounding code context (function signature, imports, types)

### 3b. Implement the request
- The content between `@@snek` and `snek@@` tells you what to do
- It may be pure instructions, or instructions mixed with existing code to modify
- Generate the appropriate code based on the content

### 3c. Apply the change
Use the Edit tool to replace the entire block (from `@@snek` line to `snek@@` line inclusive) with the generated code. Remove the markers.

### 3d. Mark todo complete
Mark this todo item as completed before moving to the next block.

## Examples

### Pure specification:
```go
func ValidateEmail(email string) error {
    // @@snek
    // Validate email format using regex
    // Return error if invalid, nil if valid
    // snek@@
}
```

### Modification request with existing code:
```python
# @@snek modify this to also accept a timeout parameter
def fetch_data(url):
    response = requests.get(url)
    return response.json()
# snek@@
```

### With file references:
```go
// @@snek implement this following the pattern in @internal/repository/user.go
func (r *OrderRepository) FindByID(id string) (*Order, error) {
// snek@@
}
```

## Important Rules

1. **Process ALL blocks** - Never skip a block
2. **Remove markers** - Final code should NOT contain `@@snek` or `snek@@`
3. **Track progress** - Use TodoWrite for each block, mark complete after applying
4. **Handle @references** - Read referenced files to understand context
5. **Match style** - Follow the surrounding code's conventions

## Start Processing

Run the parser script now and process all found blocks.
