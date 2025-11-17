# Snek CLI Tool - Complete Specification

**Version**: 0.1.0  
**Last Updated**: November 3, 2025  
**Target Audience**: CLI Tool Developers

---

## Overview

The **Snek CLI** is a command-line tool for managing `.snek/` folders. It provides:
- Session management (create, switch, list, delete)
- Chat history editing (add, remove, import, export)
- Code context management (add, remove, list)
- Validation and diagnostics
- Import/export for sharing

This document provides complete specifications for implementing the CLI tool.

---

## Table of Contents

1. [Design Goals](#design-goals)
2. [Command Structure](#command-structure)
3. [Command Specifications](#command-specifications)
4. [Output Formats](#output-formats)
5. [Error Handling](#error-handling)
6. [Interactive Mode](#interactive-mode)
7. [Configuration](#configuration)
8. [Implementation Guidelines](#implementation-guidelines)
9. [Testing](#testing)

---

## Design Goals

### User Experience

- **Intuitive**: Commands follow natural language patterns
- **Safe**: Confirm destructive operations
- **Fast**: Instant response for most operations
- **Helpful**: Clear error messages with suggestions
- **Flexible**: Multiple output formats (human, JSON, table)

### Technical Requirements

- **Cross-platform**: Works on macOS, Linux, Windows
- **Atomic operations**: All file writes are atomic
- **LSP-aware**: Respects LSP file watching
- **Standalone**: Works without LSP running
- **No dependencies**: Single binary, no runtime requirements

---

## Command Structure

### Global Syntax

```bash
snek-cli [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS...]
```

### Global Options

```bash
--help, -h         Show help message
--version, -v      Show version
--json             Output in JSON format
--quiet, -q        Suppress non-essential output
--dir <path>       Specify .snek directory (default: ./.snek)
--no-color         Disable colored output
```

### Command Categories

```
Session Management:
  session list
  session create
  session switch
  session delete
  session rename
  session show
  session clone
  session export
  session import

Chat Management:
  chat list
  chat add
  chat edit
  chat delete
  chat clear
  chat import
  chat export
  chat show

Context Management:
  context list
  context add
  context remove
  context clear
  context show
  context update

Utilities:
  init
  validate
  stats
  cleanup
  interactive
```

---

## Command Specifications

### Session Commands

#### session list

**Purpose**: List all sessions

**Syntax**:
```bash
snek-cli session list [OPTIONS]
```

**Options**:
```bash
--json          Output as JSON array
--table         Output as table (default)
--verbose, -v   Show additional details
```

**Output (default)**:
```
ID                                     Name                        Status
550e8400-e29b-41d4-a716-446655440000  Feature: User Auth          [ACTIVE]
7c9e6679-7425-40de-944b-e07fc1f90ae7  Backend Development         
a3bb189e-8bf9-3888-9912-ace4e6543002  Python Style Guide          
```

**Output (--json)**:
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Feature: User Auth",
    "version": 5,
    "active": true,
    "limits": {"max_tokens": 1600},
    "updated_at": "2025-11-03T14:30:00Z",
    "stats": {
      "messages": 12,
      "contexts": 3
    }
  },
  {
    "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
    "name": "Backend Development",
    "version": 2,
    "active": false,
    "limits": {"max_tokens": 2000},
    "updated_at": "2025-11-02T10:15:00Z",
    "stats": {
      "messages": 5,
      "contexts": 8
    }
  }
]
```

**Exit Codes**:
- `0`: Success
- `1`: No .snek directory found
- `2`: No sessions found

---

#### session create

**Purpose**: Create a new session

**Syntax**:
```bash
snek-cli session create <name> [OPTIONS]
```

**Options**:
```bash
--switch, -s        Switch to new session immediately
--max-tokens <n>    Set token limit (default: 1600)
--template <name>   Use session template
--from <uuid>       Clone from existing session
```

**Examples**:
```bash
# Basic creation
snek-cli session create "Feature: Payments"

# Create and switch
snek-cli session create "New Feature" --switch

# Create with custom token limit
snek-cli session create "Large Context" --max-tokens 3000

# Clone existing session
snek-cli session create "Feature v2" --from 550e8400-e29b-41d4-a716-446655440000
```

**Output**:
```
Created session: a3bb189e-8bf9-3888-9912-ace4e6543002
Name: Feature: Payments
To switch: snek-cli session switch a3bb189e-8bf9-3888-9912-ace4e6543002
```

**Exit Codes**:
- `0`: Success
- `1`: Invalid name
- `2`: Template not found
- `3`: Source session not found (for --from)

---

#### session switch

**Purpose**: Switch to a different session

**Syntax**:
```bash
snek-cli session switch <uuid|name> [OPTIONS]
```

**Options**:
```bash
--force, -f    Skip confirmation
```

**Examples**:
```bash
# Switch by UUID
snek-cli session switch 550e8400-e29b-41d4-a716-446655440000

# Switch by name (if unique)
snek-cli session switch "Feature: Payments"

# Switch by partial UUID
snek-cli session switch 550e8400
```

**Output**:
```
Switched to session: 550e8400-e29b-41d4-a716-446655440000
Name: Feature: Payments
Messages: 12
Contexts: 3
```

**Confirmation (if LSP running)**:
```
LSP is currently running with this session.
Switching will reload the session (may take up to 300ms).
Continue? [y/N]
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found
- `2`: User cancelled

---

#### session delete

**Purpose**: Delete a session

**Syntax**:
```bash
snek-cli session delete <uuid|name> [OPTIONS]
```

**Options**:
```bash
--force, -f    Skip confirmation
```

**Examples**:
```bash
# Delete session
snek-cli session delete 550e8400-e29b-41d4-a716-446655440000

# Force delete
snek-cli session delete "Old Feature" --force
```

**Output**:
```
⚠️  This will permanently delete session:
  ID: 550e8400-e29b-41d4-a716-446655440000
  Name: Feature: Payments
  Messages: 12
  Contexts: 3

This action cannot be undone.
Delete session? [y/N]
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found
- `2`: Cannot delete active session (switch first)
- `3`: User cancelled

---

#### session rename

**Purpose**: Rename a session

**Syntax**:
```bash
snek-cli session rename <uuid|name> <new-name> [OPTIONS]
```

**Examples**:
```bash
snek-cli session rename 550e8400 "Feature: User Authentication"
```

**Output**:
```
Renamed session 550e8400-e29b-41d4-a716-446655440000
Old name: Feature: Payments
New name: Feature: User Authentication
Version: 6 (incremented)
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found
- `2`: Invalid new name

---

#### session show

**Purpose**: Show detailed session information

**Syntax**:
```bash
snek-cli session show [uuid|name] [OPTIONS]
```

**Options**:
```bash
--json    Output as JSON
```

**Examples**:
```bash
# Show active session
snek-cli session show

# Show specific session
snek-cli session show 550e8400
```

**Output (default)**:
```
Session Details
━━━━━━━━━━━━━━━
ID:           550e8400-e29b-41d4-a716-446655440000
Name:         Feature: User Authentication
Status:       ACTIVE
Version:      6
Max Tokens:   1600
Updated:      2025-11-03 14:30:00 UTC

Statistics
━━━━━━━━━━
Chat Messages:    12 (5 user, 5 assistant, 2 system)
Code Contexts:    3
Total Context:    ~450 lines of code

Chat Preview (last 3 messages)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[user] How should I structure error handling?
[assistant] For this project, use Result types...
[user] Should I add custom error types?

Contexts
━━━━━━━━
1. src/models/user.py (lines 0-45) - User model definition
2. src/utils/validation.py (lines 10-30) - Email validator
3. src/handlers/auth.py (lines 50-100) - Auth handler example
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found

---

#### session clone

**Purpose**: Clone an existing session

**Syntax**:
```bash
snek-cli session clone <source-uuid> [new-name] [OPTIONS]
```

**Options**:
```bash
--switch, -s    Switch to cloned session
```

**Examples**:
```bash
snek-cli session clone 550e8400 "Feature v2"
snek-cli session clone 550e8400 --switch
```

**Output**:
```
Cloned session 550e8400-e29b-41d4-a716-446655440000
New ID: a3bb189e-8bf9-3888-9912-ace4e6543002
New Name: Feature v2
Copied: 12 messages, 3 contexts
```

**Exit Codes**:
- `0`: Success
- `1`: Source session not found

---

#### session export

**Purpose**: Export session to file

**Syntax**:
```bash
snek-cli session export [uuid|name] [OPTIONS] > output.snek
```

**Options**:
```bash
--include-code     Include full code contexts (default: yes)
--anonymize        Remove personal info (paths, etc.)
```

**Examples**:
```bash
# Export active session
snek-cli session export > my-session.snek

# Export specific session
snek-cli session export 550e8400 > feature-auth.snek

# Export without code
snek-cli session export --no-include-code > template.snek
```

**Output Format** (JSON):
```json
{
  "schema": 1,
  "export_version": 1,
  "exported_at": "2025-11-03T14:30:00Z",
  "session": {
    "name": "Feature: User Auth",
    "version": 6,
    "limits": {"max_tokens": 1600},
    "updated_at": "2025-11-03T14:25:00Z"
  },
  "chat": {
    "messages": [...]
  },
  "contexts": {
    "contexts": [...]
  }
}
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found

---

#### session import

**Purpose**: Import session from file

**Syntax**:
```bash
snek-cli session import <file.snek> [OPTIONS]
```

**Options**:
```bash
--switch, -s       Switch to imported session
--name <name>      Override session name
```

**Examples**:
```bash
# Import session
snek-cli session import team-standards.snek

# Import and switch
snek-cli session import feature.snek --switch

# Import with new name
snek-cli session import exported.snek --name "My Copy"
```

**Output**:
```
Importing session from team-standards.snek...
Created session: b4cc4721-4667-4b3a-9c1f-5d7f8e9a2b3c
Name: Team Backend Standards
Messages: 8
Contexts: 5
To switch: snek-cli session switch b4cc4721
```

**Exit Codes**:
- `0`: Success
- `1`: File not found
- `2`: Invalid file format
- `3`: Validation failed

---

### Chat Commands

#### chat list

**Purpose**: List all chat messages in session

**Syntax**:
```bash
snek-cli chat list [session-uuid] [OPTIONS]
```

**Options**:
```bash
--json         Output as JSON
--role <role>  Filter by role (system/user/assistant)
--last <n>     Show only last N messages
```

**Examples**:
```bash
# List all messages in active session
snek-cli chat list

# List last 5 messages
snek-cli chat list --last 5

# List only user messages
snek-cli chat list --role user
```

**Output (default)**:
```
Chat History (12 messages)
━━━━━━━━━━━━━━━━━━━━━━━━━━

 #1 [system]
 You are a helpful coding assistant specialized in Python.

 #2 [user]
 How should I structure error handling in this FastAPI project?

 #3 [assistant]
 For FastAPI projects, I recommend using custom exception handlers...
 [truncated - use 'show' to see full message]

 #4 [user]
 Should I use Pydantic for validation?

 #5 [assistant]
 Yes, Pydantic is built into FastAPI and is the recommended approach...
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found
- `2`: No messages found

---

#### chat add

**Purpose**: Add a message to chat history

**Syntax**:
```bash
snek-cli chat add <role> <content> [session-uuid] [OPTIONS]
```

**Options**:
```bash
--from-file <file>    Read content from file
--editor              Open editor to write content
```

**Examples**:
```bash
# Add simple message
snek-cli chat add system "Use snake_case for variables"

# Add from file
snek-cli chat add user --from-file architecture-notes.txt

# Add using editor
snek-cli chat add user --editor
```

**Output**:
```
Added message #13 to session 550e8400
Role: user
Preview: Use snake_case for variables
```

**Exit Codes**:
- `0`: Success
- `1`: Invalid role
- `2`: Empty content
- `3`: Session not found

---

#### chat edit

**Purpose**: Edit an existing message

**Syntax**:
```bash
snek-cli chat edit <index> [new-content] [session-uuid] [OPTIONS]
```

**Options**:
```bash
--editor    Open editor (default if no content provided)
```

**Examples**:
```bash
# Edit with new content
snek-cli chat edit 5 "Updated instructions..."

# Edit in editor
snek-cli chat edit 5 --editor
```

**Output**:
```
Editing message #5 in session 550e8400

Old content:
  Should I use Pydantic for validation?

New content:
  Should I use Pydantic for all request validation?

Updated message #5
```

**Exit Codes**:
- `0`: Success
- `1`: Invalid index
- `2`: Session not found

---

#### chat delete

**Purpose**: Delete a message

**Syntax**:
```bash
snek-cli chat delete <index> [session-uuid] [OPTIONS]
```

**Options**:
```bash
--force, -f    Skip confirmation
```

**Examples**:
```bash
snek-cli chat delete 5
snek-cli chat delete 3 --force
```

**Output**:
```
Delete message #5? [y/N]

Deleted message #5 from session 550e8400
Role: user
Content: Should I use Pydantic for validation?
```

**Exit Codes**:
- `0`: Success
- `1`: Invalid index
- `2`: User cancelled

---

#### chat clear

**Purpose**: Clear all messages

**Syntax**:
```bash
snek-cli chat clear [session-uuid] [OPTIONS]
```

**Options**:
```bash
--force, -f    Skip confirmation
```

**Output**:
```
⚠️  This will delete all 12 messages in session 550e8400
This action cannot be undone.
Clear all messages? [y/N]

Cleared all messages from session 550e8400
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found
- `2`: User cancelled

---

#### chat import

**Purpose**: Import messages from file

**Syntax**:
```bash
snek-cli chat import <file> [session-uuid] [OPTIONS]
```

**Options**:
```bash
--append       Append to existing messages (default: replace)
--format <fmt> Specify format (json/chatgpt/claude)
```

**Supported Formats**:

**1. Native JSON** (chat.json format):
```json
{
  "schema": 1,
  "messages": [
    {"role": "system", "content": "..."},
    {"role": "user", "content": "..."}
  ]
}
```

**2. ChatGPT Export** (simplified):
```json
{
  "conversations": [
    {
      "messages": [
        {"role": "user", "content": {"parts": ["..."]}},
        {"role": "assistant", "content": {"parts": ["..."]}}
      ]
    }
  ]
}
```

**3. Claude Export**:
```json
{
  "conversation_id": "...",
  "messages": [
    {"sender": "human", "text": "..."},
    {"sender": "assistant", "text": "..."}
  ]
}
```

**Examples**:
```bash
# Import and replace
snek-cli chat import exported-chat.json

# Append to existing
snek-cli chat import additional-context.json --append

# Import ChatGPT conversation
snek-cli chat import chatgpt-export.json --format chatgpt
```

**Output**:
```
Importing from exported-chat.json...
Format: Native JSON
Messages: 15
Action: Replace existing

Imported 15 messages to session 550e8400
```

**Exit Codes**:
- `0`: Success
- `1`: File not found
- `2`: Invalid format
- `3`: Session not found

---

#### chat export

**Purpose**: Export messages to file

**Syntax**:
```bash
snek-cli chat export [session-uuid] [OPTIONS] > output.json
```

**Options**:
```bash
--role <role>    Export only specific role
--last <n>       Export only last N messages
```

**Examples**:
```bash
snek-cli chat export > backup.json
snek-cli chat export --last 10 > recent.json
snek-cli chat export --role system > standards.json
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found

---

#### chat show

**Purpose**: Show full message content

**Syntax**:
```bash
snek-cli chat show <index> [session-uuid] [OPTIONS]
```

**Options**:
```bash
--json    Output as JSON
```

**Output**:
```
Message #5
━━━━━━━━━━

Role:      user
Added:     2025-11-03 14:15:00 UTC
Length:    245 characters

Content:
────────────────────────────────────────
How should I structure error handling in this FastAPI project?
Should I use custom exception classes or stick with HTTPException?
Also, how do I properly log errors without exposing sensitive data?
────────────────────────────────────────
```

**Exit Codes**:
- `0`: Success
- `1`: Invalid index
- `2`: Session not found

---

### Context Commands

#### context list

**Purpose**: List all code contexts

**Syntax**:
```bash
snek-cli context list [session-uuid] [OPTIONS]
```

**Options**:
```bash
--json           Output as JSON
--show-code      Include code in output
--language <id>  Filter by language
```

**Examples**:
```bash
snek-cli context list
snek-cli context list --show-code
snek-cli context list --language python
```

**Output (default)**:
```
Code Contexts (3)
━━━━━━━━━━━━━━━━━

 #1 [python] src/models/user.py (lines 0-45)
    User model definition
    Last updated: 2025-11-03 14:20:00

 #2 [python] src/utils/validation.py (lines 10-30)
    Email validator helper
    Last updated: 2025-11-03 14:22:15

 #3 [python] src/handlers/auth.py (lines 50-100)
    Auth handler example
    Last updated: 2025-11-03 14:25:30

Total: ~175 lines of code
```

**Output (--show-code)**:
```
 #1 [python] src/models/user.py (lines 0-45)
    User model definition
    ────────────────────────────────────────
    class User(BaseModel):
        id: int
        username: str
        email: EmailStr
    ────────────────────────────────────────
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found
- `2`: No contexts found

---

#### context add

**Purpose**: Add a code context

**Syntax**:
```bash
snek-cli context add <file> <start-line> <end-line> [description] [session-uuid] [OPTIONS]
```

**Options**:
```bash
--language <id>    Override language detection
--whole-file       Use entire file (ignore line range)
```

**Examples**:
```bash
# Add specific lines
snek-cli context add src/models/user.py 0 45 "User model"

# Add entire file
snek-cli context add src/config.py --whole-file "App config"

# Override language
snek-cli context add script.txt 0 100 "Build script" --language bash
```

**Output**:
```
Adding context from src/models/user.py...
Lines: 0-45 (45 lines)
Language: python (detected)
Description: User model

✓ Added context #4 to session 550e8400
  LSP will automatically extract and track code changes
```

**Exit Codes**:
- `0`: Success
- `1`: File not found
- `2`: Invalid line range
- `3`: Session not found

---

#### context remove

**Purpose**: Remove a code context

**Syntax**:
```bash
snek-cli context remove <index> [session-uuid] [OPTIONS]
```

**Options**:
```bash
--force, -f    Skip confirmation
```

**Examples**:
```bash
snek-cli context remove 2
snek-cli context remove 1 --force
```

**Output**:
```
Remove context #2? [y/N]

Removed context #2 from session 550e8400
File: src/utils/validation.py (lines 10-30)
Description: Email validator helper
```

**Exit Codes**:
- `0`: Success
- `1`: Invalid index
- `2`: User cancelled

---

#### context clear

**Purpose**: Clear all contexts

**Syntax**:
```bash
snek-cli context clear [session-uuid] [OPTIONS]
```

**Options**:
```bash
--force, -f    Skip confirmation
```

**Output**:
```
⚠️  This will remove all 3 contexts from session 550e8400
Continue? [y/N]

Cleared all contexts from session 550e8400
```

**Exit Codes**:
- `0`: Success
- `1`: Session not found
- `2`: User cancelled

---

#### context show

**Purpose**: Show full context details

**Syntax**:
```bash
snek-cli context show <index> [session-uuid] [OPTIONS]
```

**Options**:
```bash
--json    Output as JSON
```

**Output**:
```
Context #1
━━━━━━━━━━

File:         /Users/dev/project/src/models/user.py
Lines:        0-45 (45 lines)
Language:     python
Description:  User model definition
Last Updated: 2025-11-03 14:20:00 UTC

Code:
────────────────────────────────────────────────────────
from pydantic import BaseModel, EmailStr
from datetime import datetime

class User(BaseModel):
    id: int
    username: str
    email: EmailStr
    created_at: datetime

    class Config:
        orm_mode = True
────────────────────────────────────────────────────────

Status: ✓ Tracked by LSP (auto-updates on file changes)
```

**Exit Codes**:
- `0`: Success
- `1`: Invalid index
- `2`: Session not found

---

#### context update

**Purpose**: Update context line range

**Syntax**:
```bash
snek-cli context update <index> <start-line> <end-line> [session-uuid] [OPTIONS]
```

**Examples**:
```bash
snek-cli context update 1 0 60
```

**Output**:
```
Updating context #1 in session 550e8400

Old range: lines 0-45 (45 lines)
New range: lines 0-60 (60 lines)

✓ Updated context #1
  LSP will re-extract code on next reload
```

**Exit Codes**:
- `0`: Success
- `1`: Invalid index
- `2`: Invalid line range

---

### Utility Commands

#### init

**Purpose**: Initialize .snek directory

**Syntax**:
```bash
snek-cli init [OPTIONS]
```

**Options**:
```bash
--template <name>    Use template (minimal/full/team)
--force              Reinitialize if exists
```

**Examples**:
```bash
snek-cli init
snek-cli init --template team
```

**Output**:
```
Initializing .snek/ in current directory...

✓ Created .snek/
✓ Created .snek/sessions/
✓ Created default session (b4cc4721-4667-4b3a-9c1f-5d7f8e9a2b3c)
✓ Created active.json

Snek is ready! Run 'snek-cli session list' to get started.
```

**Exit Codes**:
- `0`: Success
- `1`: Already exists (use --force)
- `2`: Permission denied

---

#### validate

**Purpose**: Validate all files in .snek

**Syntax**:
```bash
snek-cli validate [OPTIONS]
```

**Options**:
```bash
--json    Output as JSON
--fix     Attempt to fix issues
```

**Output**:
```
Validating .snek/...

✓ active.json: Valid
✓ Session b4cc4721: Valid
  ✓ session.json: Valid
  ✓ chat.json: Valid (12 messages)
  ✓ context.json: Valid (3 contexts)
✓ Session 550e8400: Valid
  ✓ session.json: Valid
  ✓ chat.json: Valid (5 messages)
  ⚠ context.json: Warning - context #2 file not found
    File: /path/to/missing.py (will error if LSP tries to update)

Summary:
  Sessions: 2
  Issues: 1 warning
  Status: ⚠ WARNINGS
```

**Exit Codes**:
- `0`: Valid
- `1`: Errors found
- `2`: Warnings found

---

#### stats

**Purpose**: Show statistics

**Syntax**:
```bash
snek-cli stats [OPTIONS]
```

**Options**:
```bash
--json    Output as JSON
```

**Output**:
```
Snek Statistics
━━━━━━━━━━━━━━━

Sessions:         3
Active Session:   Feature: User Auth

Messages:
  Total:          27
  System:         6
  User:           11
  Assistant:      10

Contexts:
  Total:          11
  Languages:      Python (8), Go (2), TypeScript (1)
  Total Lines:    ~650

Disk Usage:       156 KB
Last Modified:    2025-11-03 14:30:00
```

**Exit Codes**:
- `0`: Success
- `1`: No .snek directory

---

#### cleanup

**Purpose**: Clean up orphaned/invalid files

**Syntax**:
```bash
snek-cli cleanup [OPTIONS]
```

**Options**:
```bash
--dry-run    Show what would be removed
--force      Skip confirmation
```

**Output**:
```
Scanning .snek/ for issues...

Found:
  ⚠ Orphaned session directory: .snek/sessions/old-uuid/ (not in active.json)
  ⚠ Backup file: .snek/active.json.bak
  ⚠ Temp file: .snek/sessions/abc/.session.json.tmp

Remove these files? [y/N]

Cleaned up 3 items
Freed: 24 KB
```

**Exit Codes**:
- `0`: Success
- `1`: No issues found
- `2`: User cancelled

---

#### interactive

**Purpose**: Launch interactive TUI

**Syntax**:
```bash
snek-cli interactive
```

**Features**:
- Browse sessions with arrow keys
- Switch sessions with Enter
- Edit chat messages inline
- Manage contexts visually
- Live validation
- Keyboard shortcuts

**UI Layout**:
```
┌─ Snek CLI ──────────────────────────────────────────────┐
│                                                          │
│ Sessions (3)                    Active: Feature: Auth    │
│ ┌────────────────────────────────────────────────────┐  │
│ │ ● Feature: User Auth                          [12]  │  │
│ │   Backend Development                          [5]  │  │
│ │   Python Style Guide                           [8]  │  │
│ └────────────────────────────────────────────────────┘  │
│                                                          │
│ Chat History (12 messages)                              │
│ ┌────────────────────────────────────────────────────┐  │
│ │ [system] You are a helpful coding assistant...     │  │
│ │ [user] How should I structure error handling?      │  │
│ │ [assistant] For this project, use Result types...  │  │
│ └────────────────────────────────────────────────────┘  │
│                                                          │
│ [s]witch [e]dit [d]elete [a]dd [q]uit                  │
└──────────────────────────────────────────────────────────┘
```

**Exit Codes**:
- `0`: Normal exit
- `1`: Error launching TUI

---

## Output Formats

### Human-Readable (Default)

- **Colored**: Use ANSI colors for clarity
- **Tables**: Use box drawing characters
- **Truncation**: Truncate long content with "..." and hint to use `show`
- **Symbols**: Use ✓, ✗, ⚠ for status

### JSON Format

**Flag**: `--json`

**All JSON output follows this structure**:
```json
{
  "success": true,
  "data": { ... },
  "metadata": {
    "timestamp": "2025-11-03T14:30:00Z",
    "command": "session list",
    "version": "0.1.0"
  }
}
```

**Error Format**:
```json
{
  "success": false,
  "error": {
    "code": "SESSION_NOT_FOUND",
    "message": "Session 550e8400 not found",
    "suggestion": "Run 'snek-cli session list' to see available sessions"
  },
  "metadata": { ... }
}
```

### Table Format

**Flag**: `--table` (default for list commands)

Uses Unicode box drawing:
```
┌──────────────┬─────────────┬────────┐
│ ID           │ Name        │ Status │
├──────────────┼─────────────┼────────┤
│ 550e8400     │ Feature     │ ACTIVE │
│ 7c9e6679     │ Backend     │        │
└──────────────┴─────────────┴────────┘
```

---

## Error Handling

### Error Categories

**1. User Errors** (Exit Code 1):
- Invalid arguments
- File not found
- Session not found

**2. System Errors** (Exit Code 2):
- Permission denied
- Disk full
- Invalid JSON

**3. User Cancellation** (Exit Code 3):
- User pressed Ctrl+C
- User answered "No" to confirmation

### Error Messages

**Format**:
```
Error: <clear description>
  Reason: <technical reason>
  Suggestion: <what to do>
```

**Example**:
```
Error: Session not found
  Reason: No session with ID 550e8400 exists
  Suggestion: Run 'snek-cli session list' to see available sessions
```

### Warnings

Non-fatal issues:
```
Warning: Context file not found
  Context #2 references /path/to/missing.py
  This will cause errors if LSP tries to update it
  Suggestion: Remove context or restore file
```

---

## Interactive Mode

### Key Bindings

```
Global:
  q, Esc       Quit
  h, ?         Help
  j, ↓         Down
  k, ↑         Up
  /            Search
  :            Command mode

Session View:
  Enter        Show details
  s            Switch session
  c            Create session
  d            Delete session
  r            Rename session

Chat View:
  a            Add message
  e            Edit message
  d            Delete message
  x            Clear all

Context View:
  a            Add context
  d            Remove context
  v            View code
  x            Clear all
```

### Themes

Support multiple color schemes:
- `default`: Light background, dark text
- `dark`: Dark background, light text
- `solarized`: Solarized theme
- `none`: No colors (for scripts)

Configure in `~/.config/snek/config.toml`:
```toml
[display]
theme = "dark"
unicode = true
```

---

## Configuration

### Config File

**Location**: `~/.config/snek/config.toml`

**Format**:
```toml
[default]
# Default session settings
max_tokens = 1600

[api]
# API configuration (optional, usually from env)
# key = "sk-..."
# base_url = "https://api.openai.com/v1"

[editor]
# Editor for --editor flag
command = "vim"
args = []

[display]
# Display settings
theme = "dark"
unicode = true
colors = true
table_style = "rounded"

[paths]
# Custom paths
snek_dir = ".snek"  # Relative to current directory

[interactive]
# Interactive mode settings
refresh_rate = 100  # ms
vim_mode = false
```

---

## Implementation Guidelines

### Language Recommendations

**Recommended**: Rust, Go, or Python

**Rust** (Recommended):
- Pros: Fast, single binary, cross-platform, type-safe
- Cons: Longer compile times
- Libraries: `clap` (CLI), `serde_json` (JSON), `tui-rs` (TUI)

**Go**:
- Pros: Fast compilation, easy concurrency, single binary
- Cons: Verbose error handling
- Libraries: `cobra` (CLI), `bubbletea` (TUI)

**Python**:
- Pros: Rapid development, good libraries
- Cons: Requires Python runtime
- Libraries: `click` (CLI), `rich` (formatting), `textual` (TUI)

### Architecture

```
snek-cli/
├── src/
│   ├── main.rs/go/py          # Entry point
│   ├── commands/               # Command implementations
│   │   ├── session.rs
│   │   ├── chat.rs
│   │   ├── context.rs
│   │   └── utils.rs
│   ├── fs/                     # File system operations
│   │   ├── reader.rs
│   │   ├── writer.rs
│   │   └── validator.rs
│   ├── types/                  # Data structures
│   │   ├── session.rs
│   │   ├── chat.rs
│   │   └── context.rs
│   ├── output/                 # Output formatters
│   │   ├── human.rs
│   │   ├── json.rs
│   │   └── table.rs
│   └── interactive/            # TUI mode
│       ├── app.rs
│       └── widgets/
└── tests/
    ├── integration/
    └── unit/
```

### Key Principles

1. **Atomic Writes**: Always use tmp + rename
2. **Validation**: Validate before writing
3. **Error Context**: Provide helpful error messages
4. **LSP Aware**: Don't corrupt files LSP is watching
5. **Idempotent**: Running same command twice = same result
6. **Fast**: Most operations < 100ms

### File Operations Pattern

```rust
// Atomic write pattern
fn write_json(path: &Path, data: &impl Serialize) -> Result<()> {
    let tmp = NamedTempFile::new_in(path.parent().unwrap())?;
    serde_json::to_writer_pretty(&tmp, data)?;
    tmp.persist(path)?;
    Ok(())
}

// Read with validation
fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path)?;
    let data: T = serde_json::from_str(&content)?;
    // Validate here
    Ok(data)
}
```

---

## Testing

### Unit Tests

Test each command independently:

```rust
#[test]
fn test_session_create() {
    let temp_dir = TempDir::new()?;
    let cli = SnekCli::new(temp_dir.path());
    
    let result = cli.session_create("Test Session", None)?;
    
    assert!(result.id.is_valid_uuid());
    assert_eq!(result.name, "Test Session");
    assert!(temp_dir.path().join(".snek/sessions").join(&result.id).exists());
}
```

### Integration Tests

Test full workflows:

```rust
#[test]
fn test_full_workflow() {
    let cli = setup_cli();
    
    // Create session
    let session = cli.session_create("Test")?;
    
    // Add chat message
    cli.chat_add(&session.id, "user", "Hello")?;
    
    // Add context
    cli.context_add(&session.id, "test.py", 0, 10, None)?;
    
    // Export
    let export = cli.session_export(&session.id)?;
    
    // Import
    let imported = cli.session_import(&export)?;
    
    assert_eq!(imported.name, session.name);
}
```

### Manual Testing Checklist

- [ ] Create, list, switch, delete sessions
- [ ] Add, edit, delete chat messages
- [ ] Add, remove, list contexts
- [ ] Export and import sessions
- [ ] Validate corrupted files
- [ ] Test with LSP running
- [ ] Test interactive mode
- [ ] Test on Windows/Mac/Linux

---

## Appendix: Examples

### Example Session Template (minimal)

```json
{
  "schema": 1,
  "session": {
    "name": "Minimal Template",
    "limits": {"max_tokens": 1600}
  },
  "chat": {
    "messages": []
  },
  "contexts": {
    "contexts": []
  }
}
```

### Example Session Template (team)

```json
{
  "schema": 1,
  "session": {
    "name": "Team Backend Standards",
    "limits": {"max_tokens": 2000}
  },
  "chat": {
    "messages": [
      {
        "role": "system",
        "content": "Follow company coding standards: snake_case for Python, camelCase for JavaScript, PascalCase for classes"
      },
      {
        "role": "system",
        "content": "Always add type hints in Python and JSDoc in JavaScript"
      },
      {
        "role": "system",
        "content": "Use explicit error handling - no bare except or unchecked errors"
      }
    ]
  },
  "contexts": {
    "contexts": []
  }
}
```

---

**End of CLI Tool Specification**

For complete file format details, see `SNEK_FOLDER_GUIDE.md`.

