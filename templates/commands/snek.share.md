---
description: Share conversation history with snek LSP
allowed-tools: Read(*), Write(*), Bash(*)
---

# Share Conversation with Snek

Export your current Claude conversation to snek's session system so snek can understand the full context of your work.

---

## STEP 1: Find the Active Session

First, read `.snek/active.json` to find the current active session:

```bash
cat .snek/active.json
```

This file contains:
```json
{
  "schema": 1,
  "id": "session-uuid-here",
  "path": "sessions/session-uuid-here"
}
```

The **session directory** is `.snek/sessions/{id}/` where `{id}` is the UUID from the `id` field.

**Example**: If `id` is `"b4c12a8f-5a92-4e92-8d3c-7f9a1c2e5b8a"`, the session directory is:
```
.snek/sessions/b4c12a8f-5a92-4e92-8d3c-7f9a1c2e5b8a/
```

---

## STEP 2: Navigate to Session Context Folder

The session directory structure is:
```
.snek/sessions/{session-id}/
├── session.json           # Session metadata (don't edit)
├── code_snippets.json     # Code file references (you'll edit this)
└── context/               # Markdown context files (you'll add files here)
    ├── architecture.md
    ├── conventions.md
    └── ... (your new files)
```

**Your target locations**:
- **Markdown context**: `.snek/sessions/{session-id}/context/`
- **Code references**: `.snek/sessions/{session-id}/code_snippets.json`

---

## STEP 3: Create/Update Markdown Context Files

**Location**: `.snek/sessions/{session-id}/context/`

### 3A. Check Existing Files

List all existing markdown files:
```bash
ls .snek/sessions/{session-id}/context/
```

Read each file to see what context is already documented. **DO NOT duplicate information** that's already there.

### 3B. Create New Context File(s)

Create a new markdown file with a descriptive name that reflects this conversation. Good names:
- `claude_conversation_2025_11_07.md`
- `lsp_implementation_discussion.md`
- `session_management_context.md`

**File naming rules**:
- Use lowercase with underscores
- Be descriptive and specific
- Files are read **alphabetically**, so prefix with numbers if order matters (e.g., `01_overview.md`, `02_details.md`)

### 3C. File Content Structure

Your markdown file should contain:

```markdown
# [Conversation Topic] - [Date]

## Session Overview
Brief 1-2 sentence summary of what this conversation was about.

## User's Objectives
- Primary goal 1
- Primary goal 2
- Any specific requirements or constraints mentioned

## Conversation Summary

### User Request 1
[What the user asked for]

**Response**:
[What you (Claude) did/said - key points only, not full verbatim text]

**Outcome**:
[What was accomplished or decided]

---

### User Request 2
[Next question/request]

**Response**:
[Your response summary]

**Outcome**:
[Result or decision]

---

[Continue for all major exchanges...]

## Key Decisions & Rationale
- **Decision**: [What was decided]
  - **Why**: [Reasoning behind it]
  - **Alternatives considered**: [What we didn't choose and why]

## Technical Details
[Any specific technical information, configurations, API details, etc.]

## Known Issues / Limitations
- [Issue 1]: [Description and why it matters]
- [Limitation 1]: [What doesn't work and potential workarounds]

## Current State
[Where things stand at the end of this conversation]

## Next Steps / Open Questions
- [ ] [Action item 1]
- [ ] [Action item 2]
- [ ] [Open question that needs answering]
```

**Important guidelines**:
- Be **concise but complete** - include all relevant technical details
- Focus on **decisions, rationale, and context** that snek needs to understand
- Include **warnings and gotchas** you mentioned to the user
- Document **what didn't work** and why (failed approaches are valuable context)
- Use **clear section headers** for easy scanning
- **Don't** include full code blocks - those go in code_snippets.json

---

## STEP 4: Update code_snippets.json

**Location**: `.snek/sessions/{session-id}/code_snippets.json`

### 4A. Read Existing File

First, read the current `code_snippets.json`:
```bash
cat .snek/sessions/{session-id}/code_snippets.json
```

It looks like this:
```json
{
  "schema": 1,
  "snippets": [
    {
      "uri": "file:///absolute/path/to/src/model.rs",
      "start_line": 0,
      "end_line": 100,
      "language_id": "rust",
      "description": "Core data models",
      "last_modified": "2025-11-02T09:15:00Z"
    }
  ]
}
```

### 4B. Add New Code References

For **every file** that was:
- Read during the conversation
- Modified or created
- Discussed or mentioned
- Relevant to understanding this conversation

Add an entry to the `snippets` array.

**Field requirements**:

1. **`uri`** (required, string):
   - **MUST** start with `file://`
   - **MUST** be an **absolute path** to the file
   - Example: `"file:///Users/user/projects/snek_lsp/src/lib.rs"`
   - Get the absolute path with: `realpath src/lib.rs`

2. **`start_line`** (required, number):
   - **Zero-indexed** line number where snippet starts
   - Use `0` to start from the beginning of the file
   - To include lines 10-50, use `start_line: 9` (because 0-indexed)

3. **`end_line`** (required, number):
   - **Exclusive** end line (not included in snippet)
   - To include lines 10-50, use `end_line: 51` (because exclusive)
   - Use the total line count to include the whole file

4. **`language_id`** (required, string):
   - Language identifier: `"rust"`, `"python"`, `"javascript"`, `"typescript"`, `"markdown"`, etc.
   - Must match VS Code language identifiers

5. **`description`** (optional, string):
   - Brief description of **why this snippet matters** to the conversation
   - Examples:
     - `"Session loading logic discussed in conversation"`
     - `"Modified to fix the bug user reported"`
     - `"User asked about how this struct works"`

6. **`last_modified`** (required, string):
   - RFC3339 timestamp of when the file was last modified
   - Get it with: `date -r filename +"%Y-%m-%dT%H:%M:%SZ"` (macOS) or `date -r filename --iso-8601=seconds` (Linux)
   - Or just use current time: `date -u +"%Y-%m-%dT%H:%M:%SZ"`

### 4C. Complete Example

Here's a complete example with multiple snippets:

```json
{
  "schema": 1,
  "snippets": [
    {
      "uri": "file:///Users/melshakobyan/Desktop/projects/snek_ecosystem/snek_lsp/src/session_io.rs",
      "start_line": 0,
      "end_line": 200,
      "language_id": "rust",
      "description": "Session I/O functions - discussed how active.json is loaded",
      "last_modified": "2025-11-07T10:30:00Z"
    },
    {
      "uri": "file:///Users/melshakobyan/Desktop/projects/snek_ecosystem/snek_lsp/src/snapshot.rs",
      "start_line": 40,
      "end_line": 60,
      "language_id": "rust",
      "description": "ContextSnapshot struct definition - explained to user",
      "last_modified": "2025-11-07T10:30:00Z"
    },
    {
      "uri": "file:///Users/melshakobyan/Desktop/projects/snek_ecosystem/snek_lsp/src/model.rs",
      "start_line": 133,
      "end_line": 169,
      "language_id": "rust",
      "description": "Markdown context reading logic - user asked about this",
      "last_modified": "2025-11-07T10:30:00Z"
    }
  ]
}
```

### 4D. How to Get File Information

To add a file reference, you need:

1. **Absolute path**:
   ```bash
   realpath src/session_io.rs
   # Output: /Users/user/projects/snek_lsp/src/session_io.rs
   # Use as: file:///Users/user/projects/snek_lsp/src/session_io.rs
   ```

2. **Line count** (for end_line):
   ```bash
   wc -l src/session_io.rs
   # Output: 200 src/session_io.rs
   # Use as: end_line: 200
   ```

3. **Last modified timestamp**:
   ```bash
   # macOS:
   date -r src/session_io.rs +"%Y-%m-%dT%H:%M:%SZ"

   # Linux:
   date -r src/session_io.rs --iso-8601=seconds

   # Or just use current time:
   date -u +"%Y-%m-%dT%H:%M:%SZ"
   ```

### 4E. Important Rules

- **DO NOT remove** existing snippets unless they're truly obsolete
- **ADD** new snippets for files mentioned in this conversation
- **UPDATE** descriptions if you discussed a file that's already in the list
- **Keep the `schema: 1`** field at the top
- **Preserve valid JSON** - use a JSON validator if unsure

---

## STEP 5: Verify Your Work

After updating the files, verify everything is correct:

1. **Check JSON syntax**:
   ```bash
   python3 -m json.tool .snek/sessions/{session-id}/code_snippets.json
   ```
   If this fails, you have a JSON syntax error. Fix it.

2. **Check file paths exist**:
   ```bash
   # For each URI in code_snippets.json, verify the file exists
   # Remove the "file://" prefix and check:
   ls /Users/user/projects/snek_lsp/src/model.rs
   ```

3. **Read your markdown context**:
   ```bash
   cat .snek/sessions/{session-id}/context/your_new_file.md
   ```
   Make sure it's clear and complete.

---

## STEP 6: Confirm to User

After completing all steps, tell the user:

1. Which markdown file(s) you created/updated in the context folder
2. How many code snippets you added to code_snippets.json
3. What files are now referenced for snek's context
4. A brief summary of what knowledge you shared

Example message:
```
✓ Shared conversation with snek:

Context files updated:
  - .snek/sessions/b4c12a8f/context/claude_session_2025_11_07.md (new)
  - 1,234 words of conversation context

Code references added:
  - src/session_io.rs (lines 0-200) - Session loading logic
  - src/snapshot.rs (lines 40-60) - ContextSnapshot struct
  - src/model.rs (lines 133-169) - Markdown reading
  - 3 total snippets in code_snippets.json

Snek now has full context of our discussion about session management.
```

---

## Quick Reference: File Paths

- **Active session**: `.snek/active.json`
- **Session metadata**: `.snek/sessions/{id}/session.json` (don't edit)
- **Code references**: `.snek/sessions/{id}/code_snippets.json` (edit this)
- **Context markdown**: `.snek/sessions/{id}/context/*.md` (create files here)

---

## Troubleshooting

**Q: code_snippets.json doesn't exist**
- Create it with the schema structure:
  ```json
  {
    "schema": 1,
    "snippets": []
  }
  ```

**Q: The context/ folder doesn't exist**
- Create it:
  ```bash
  mkdir -p .snek/sessions/{session-id}/context
  ```

**Q: I don't know the absolute path to a file**
- Use `realpath`:
  ```bash
  realpath relative/path/to/file.rs
  ```

**Q: My JSON is invalid**
- Validate with:
  ```bash
  python3 -m json.tool file.json
  ```
- Common mistakes:
  - Trailing commas (not allowed in JSON)
  - Missing quotes around strings
  - Missing commas between array elements

---

## Remember

- **Check existing files first** - don't duplicate context
- **Be thorough** - include all technical decisions and rationale
- **Use absolute paths** in URIs with `file://` prefix
- **Validate JSON** before finishing
- **Describe why** each code snippet matters to the conversation
