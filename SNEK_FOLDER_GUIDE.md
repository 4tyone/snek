# .snek Folder - Complete Reference Guide

**Version**: 0.1.0  
**Last Updated**: November 3, 2025  
**Target Audience**: CLI Tool Developers, Extension Developers, Advanced Users

---

## Overview

The `.snek/` directory is the central configuration hub for Snek LSP. It stores:
- **Sessions**: Isolated workspaces with their own chat history and code contexts
- **Active Session**: Pointer to currently active session
- **Chat History**: Conversation context that influences AI completions
- **Code Contexts**: Snippets from other files included in AI prompts

This guide provides complete specifications for anyone building tools to manage `.snek/` folders.

---

## Table of Contents

1. [Directory Structure](#directory-structure)
2. [File Specifications](#file-specifications)
3. [Session Management](#session-management)
4. [File Operations](#file-operations)
5. [Validation Rules](#validation-rules)
6. [Use Cases & Workflows](#use-cases--workflows)
7. [CLI Tool Requirements](#cli-tool-requirements)
8. [Security Considerations](#security-considerations)
9. [Migration & Versioning](#migration--versioning)

---

## Directory Structure

### Complete Layout

```
.snek/
├── active.json                           # Active session pointer
└── sessions/                             # Session storage
    ├── <session-uuid-1>/
    │   ├── session.json                  # Session metadata
    │   ├── chat.json                     # Chat history
    │   └── context.json                  # Code contexts
    ├── <session-uuid-2>/
    │   ├── session.json
    │   ├── chat.json
    │   └── context.json
    └── <session-uuid-N>/
        ├── session.json
        ├── chat.json
        └── context.json
```

### File Locations

| File | Path | Purpose | Watch |
|------|------|---------|-------|
| `active.json` | `.snek/active.json` | Points to active session | ✅ Yes |
| `session.json` | `.snek/sessions/<uuid>/session.json` | Session metadata | ✅ Yes |
| `chat.json` | `.snek/sessions/<uuid>/chat.json` | Chat messages | ✅ Yes |
| `context.json` | `.snek/sessions/<uuid>/context.json` | Code contexts | ✅ Yes |

### Directory Permissions

```bash
.snek/           755 (rwxr-xr-x)
sessions/        755 (rwxr-xr-x)
<uuid>/          755 (rwxr-xr-x)
*.json           644 (rw-r--r--)
```

---

## File Specifications

### 1. active.json

**Purpose**: Points to the currently active session

**Location**: `.snek/active.json`

**Format**:
```json
{
  "schema": 1,
  "id": "f64fd03f-dbf5-4f43-aa5d-c919f92cc419",
  "path": "sessions/f64fd03f-dbf5-4f43-aa5d-c919f92cc419"
}
```

#### Fields

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| `schema` | integer | ✅ Yes | Schema version (currently 1) | Must be 1 |
| `id` | string | ✅ Yes | UUID of active session | Valid UUID v4 |
| `path` | string | ✅ Yes | Relative path to session dir | Must start with `sessions/` |

#### Constraints

- `id` must match the directory name in `path`
- Referenced session directory must exist
- File must be valid JSON
- Must be atomically written (use tmp + rename)

#### Example Valid Files

```json
{
  "schema": 1,
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "path": "sessions/550e8400-e29b-41d4-a716-446655440000"
}
```

#### Example Invalid Files

```json
// ❌ Missing required field
{
  "schema": 1,
  "path": "sessions/abc-123"
}

// ❌ Invalid UUID format
{
  "schema": 1,
  "id": "not-a-uuid",
  "path": "sessions/not-a-uuid"
}

// ❌ Path doesn't match ID
{
  "schema": 1,
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "path": "sessions/different-uuid"
}
```

---

### 2. session.json

**Purpose**: Stores session metadata and configuration

**Location**: `.snek/sessions/<uuid>/session.json`

**Format**:
```json
{
  "schema": 1,
  "id": "f64fd03f-dbf5-4f43-aa5d-c919f92cc419",
  "name": "Feature: User Authentication",
  "version": 5,
  "limits": {
    "max_tokens": 1600
  },
  "updated_at": "2025-11-03T14:30:00Z"
}
```

#### Fields

| Field | Type | Required | Description | Default | Validation |
|-------|------|----------|-------------|---------|------------|
| `schema` | integer | ✅ Yes | Schema version | - | Must be 1 |
| `id` | string | ✅ Yes | Session UUID | - | Valid UUID v4 |
| `name` | string | ✅ Yes | Human-readable name | `"default"` | 1-100 chars |
| `version` | integer | ✅ Yes | Incremental version | `0` | >= 0 |
| `limits` | object | ✅ Yes | Token limits | - | See below |
| `limits.max_tokens` | integer | ✅ Yes | Max completion tokens | `1600` | 1-4096 |
| `updated_at` | string | ✅ Yes | ISO 8601 timestamp | Now | Valid RFC3339 |

#### Version Behavior

- Starts at `0` for new sessions
- Increment by 1 on each modification
- Used by LSP for cache invalidation
- Never decrement or skip numbers

#### Name Guidelines

- **Good**: `"Feature: User Auth"`, `"Main Development"`, `"Python Style Guide"`
- **Avoid**: Empty strings, only whitespace, very long names

#### Token Limits

Common values:
- **Fast completions**: `512` - `1000` tokens
- **Balanced**: `1600` tokens (default)
- **Detailed**: `2000` - `4096` tokens

Higher tokens = slower response, higher cost, more detailed suggestions.

#### Example Files

**Minimal Session**:
```json
{
  "schema": 1,
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "default",
  "version": 0,
  "limits": {
    "max_tokens": 1600
  },
  "updated_at": "2025-11-03T12:00:00Z"
}
```

**Production Session**:
```json
{
  "schema": 1,
  "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "name": "Backend API Development",
  "version": 42,
  "limits": {
    "max_tokens": 2000
  },
  "updated_at": "2025-11-03T14:30:15.123Z"
}
```

---

### 3. chat.json

**Purpose**: Stores conversation history that provides context to AI

**Location**: `.snek/sessions/<uuid>/chat.json`

**Format**:
```json
{
  "schema": 1,
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful coding assistant specialized in Python."
    },
    {
      "role": "user",
      "content": "How should I structure error handling in this FastAPI project?"
    },
    {
      "role": "assistant",
      "content": "For FastAPI projects, I recommend using custom exception handlers and HTTPException for API errors..."
    },
    {
      "role": "user",
      "content": "Should I use Pydantic for validation?"
    },
    {
      "role": "assistant",
      "content": "Yes, Pydantic is built into FastAPI and is the recommended approach for request/response validation..."
    }
  ]
}
```

#### Fields

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| `schema` | integer | ✅ Yes | Schema version | Must be 1 |
| `messages` | array | ✅ Yes | List of messages | Can be empty |

#### Message Object

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| `role` | string | ✅ Yes | Message sender | `"system"`, `"user"`, or `"assistant"` |
| `content` | string | ✅ Yes | Message text | Non-empty string |

#### Role Descriptions

**system**:
- Sets overall behavior and constraints
- Should be first message (if present)
- Examples: coding standards, language preferences, style guides
- Multiple system messages are allowed

**user**:
- Questions, requests, or instructions from developer
- Can be conversation starters or follow-ups

**assistant**:
- AI responses to user messages
- Usually follows user messages (but not required)
- Contains explanations, code examples, suggestions

#### Message Order

Messages are processed in array order. Typical patterns:

**Pattern 1: System + Conversation**
```json
{
  "messages": [
    {"role": "system", "content": "Use TypeScript strict mode"},
    {"role": "user", "content": "How to define interfaces?"},
    {"role": "assistant", "content": "Use interface keyword..."}
  ]
}
```

**Pattern 2: Multiple Systems**
```json
{
  "messages": [
    {"role": "system", "content": "Use snake_case for Python"},
    {"role": "system", "content": "Prefer explicit error handling"},
    {"role": "system", "content": "Add type hints to all functions"}
  ]
}
```

**Pattern 3: Ongoing Conversation**
```json
{
  "messages": [
    {"role": "user", "content": "Explain async/await"},
    {"role": "assistant", "content": "Async/await is..."},
    {"role": "user", "content": "Show me an example"},
    {"role": "assistant", "content": "Here's an example: async def..."},
    {"role": "user", "content": "What about error handling?"},
    {"role": "assistant", "content": "For async errors, use try/except..."}
  ]
}
```

#### Size Considerations

- **Recommended**: 5-20 messages for optimal context
- **Maximum**: Limited only by AI model context window
- **Trade-off**: More messages = more context but slower completions

#### Use Cases

1. **Team Coding Standards**
   ```json
   {
     "messages": [
       {"role": "system", "content": "Follow Google Python Style Guide"},
       {"role": "system", "content": "Use descriptive variable names"},
       {"role": "system", "content": "Add docstrings to all public functions"}
     ]
   }
   ```

2. **Project Architecture Decisions**
   ```json
   {
     "messages": [
       {"role": "user", "content": "We're using hexagonal architecture"},
       {"role": "assistant", "content": "Understood. I'll suggest code following ports and adapters pattern..."},
       {"role": "user", "content": "Domain logic goes in core/"},
       {"role": "assistant", "content": "Got it. Core layer for business logic, adapters for external systems..."}
     ]
   }
   ```

3. **Language-Specific Preferences**
   ```json
   {
     "messages": [
       {"role": "system", "content": "For Rust: prefer ? operator over unwrap()"},
       {"role": "system", "content": "Use Result<T, E> for error handling"},
       {"role": "system", "content": "Add #[must_use] to functions returning Result"}
     ]
   }
   ```

4. **Imported Conversations**
   - Export conversations from ChatGPT, Claude, etc.
   - Convert to chat.json format
   - Provides consistent AI behavior based on past discussions

---

### 4. context.json

**Purpose**: Code snippets from other files included in AI prompts

**Location**: `.snek/sessions/<uuid>/context.json`

**Format**:
```json
{
  "schema": 1,
  "contexts": [
    {
      "uri": "file:///Users/dev/project/src/models/user.py",
      "start_line": 0,
      "end_line": 45,
      "language_id": "python",
      "code": "class User(BaseModel):\n    id: int\n    username: str\n    email: EmailStr\n    created_at: datetime\n\n    class Config:\n        orm_mode = True",
      "description": "User model with Pydantic validation",
      "last_modified": "2025-11-03T14:25:30Z"
    },
    {
      "uri": "file:///Users/dev/project/src/utils/auth.py",
      "start_line": 10,
      "end_line": 35,
      "language_id": "python",
      "code": "def verify_token(token: str) -> Optional[User]:\n    try:\n        payload = jwt.decode(token, SECRET_KEY)\n        return get_user_by_id(payload['user_id'])\n    except JWTError:\n        return None",
      "description": "JWT token verification function",
      "last_modified": "2025-11-03T14:26:00Z"
    }
  ]
}
```

#### Fields

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| `schema` | integer | ✅ Yes | Schema version | Must be 1 |
| `contexts` | array | ✅ Yes | List of code contexts | Can be empty |

#### Context Object

| Field | Type | Required | Auto-Populated | Description | Validation |
|-------|------|----------|----------------|-------------|------------|
| `uri` | string | ✅ Yes | No | File URI | Valid `file://` URI |
| `start_line` | integer | ✅ Yes | No | First line (0-indexed) | >= 0 |
| `end_line` | integer | ✅ Yes | No | Last line (exclusive) | > start_line |
| `language_id` | string | ✅ Yes | No | Language identifier | Valid LSP language ID |
| `code` | string | ✅ Yes | **✅ Yes** | Extracted code | Auto-filled by LSP |
| `description` | string | No | No | Human description | 1-200 chars |
| `last_modified` | string | ✅ Yes | **✅ Yes** | ISO 8601 timestamp | Valid RFC3339 |

#### Auto-Population Behavior

When you add a context entry:

**1. You provide**:
```json
{
  "uri": "file:///path/to/file.py",
  "start_line": 10,
  "end_line": 30,
  "language_id": "python",
  "description": "Helper function"
}
```

**2. LSP automatically fills**:
```json
{
  "uri": "file:///path/to/file.py",
  "start_line": 10,
  "end_line": 30,
  "language_id": "python",
  "code": "def helper(x, y):\n    return x + y",
  "description": "Helper function",
  "last_modified": "2025-11-03T14:30:00Z"
}
```

**3. LSP watches the source file**:
- When `file.py` changes, LSP re-extracts lines 10-30
- Updates `code` field with new content
- Updates `last_modified` timestamp
- No manual intervention needed

#### Language IDs

Use standard LSP language identifiers:

| Language | Language ID |
|----------|-------------|
| Python | `python` |
| JavaScript | `javascript` |
| TypeScript | `typescript` |
| Rust | `rust` |
| Go | `go` |
| Java | `java` |
| C++ | `cpp` |
| C | `c` |
| C# | `csharp` |
| Ruby | `ruby` |
| PHP | `php` |
| Swift | `swift` |
| Kotlin | `kotlin` |
| SQL | `sql` |
| Shell | `shellscript` |
| HTML | `html` |
| CSS | `css` |
| JSON | `json` |
| YAML | `yaml` |
| Markdown | `markdown` |

#### URI Format

Must be absolute `file://` URIs:

**✅ Valid**:
```
file:///Users/dev/project/src/main.py
file:///home/user/workspace/app.js
file:///C:/Projects/app/index.ts
```

**❌ Invalid**:
```
src/main.py                    (not absolute)
/Users/dev/project/main.py     (missing file:// scheme)
https://example.com/file.py    (wrong scheme)
```

#### Line Ranges

- **0-indexed**: First line is 0
- **Exclusive end**: `end_line` is not included
- **Examples**:
  - Lines 0-10 = first 10 lines (0 through 9)
  - Lines 5-6 = just line 5
  - Lines 0-1 = just line 0

#### Size Recommendations

- **Ideal**: 10-50 lines per context
- **Maximum**: 200 lines per context
- **Total contexts**: 3-10 for optimal performance

More context = richer suggestions but longer prompt = slower response.

#### Use Cases

**1. Reference Model Definitions**
```json
{
  "uri": "file:///project/models/user.py",
  "start_line": 0,
  "end_line": 30,
  "language_id": "python",
  "description": "User model schema"
}
```
*Use when*: Writing API endpoints that need User model

**2. Include Utility Functions**
```json
{
  "uri": "file:///project/utils/validators.py",
  "start_line": 15,
  "end_line": 40,
  "language_id": "python",
  "description": "Email validation helper"
}
```
*Use when*: Writing code that validates emails

**3. Show Test Examples**
```json
{
  "uri": "file:///project/tests/test_auth.py",
  "start_line": 20,
  "end_line": 45,
  "language_id": "python",
  "description": "Example authentication test"
}
```
*Use when*: Writing new tests in similar style

**4. Reference Constants/Config**
```json
{
  "uri": "file:///project/config.py",
  "start_line": 0,
  "end_line": 20,
  "language_id": "python",
  "description": "App configuration constants"
}
```
*Use when*: Using configuration values in code

**5. Show Related Implementation**
```json
{
  "uri": "file:///project/handlers/user_handler.go",
  "start_line": 50,
  "end_line": 100,
  "language_id": "go",
  "description": "User handler implementation pattern"
}
```
*Use when*: Writing similar handlers for other entities

---

## Session Management

### Creating a New Session

**Steps**:

1. Generate UUID v4
2. Create session directory
3. Create session.json with metadata
4. Create empty chat.json
5. Create empty context.json
6. (Optional) Update active.json to switch to new session

**Example Script**:
```bash
#!/bin/bash
SESSION_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
SESSION_DIR=".snek/sessions/$SESSION_ID"

# Create directory
mkdir -p "$SESSION_DIR"

# Create session.json
cat > "$SESSION_DIR/session.json" <<EOF
{
  "schema": 1,
  "id": "$SESSION_ID",
  "name": "New Session",
  "version": 0,
  "limits": {
    "max_tokens": 1600
  },
  "updated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

# Create empty chat.json
cat > "$SESSION_DIR/chat.json" <<EOF
{
  "schema": 1,
  "messages": []
}
EOF

# Create empty context.json
cat > "$SESSION_DIR/context.json" <<EOF
{
  "schema": 1,
  "contexts": []
}
EOF

echo "Created session: $SESSION_ID"
```

### Switching Sessions

**Method 1: Update active.json**

```bash
#!/bin/bash
TARGET_SESSION="550e8400-e29b-41d4-a716-446655440000"

cat > .snek/active.json <<EOF
{
  "schema": 1,
  "id": "$TARGET_SESSION",
  "path": "sessions/$TARGET_SESSION"
}
EOF

echo "Switched to session: $TARGET_SESSION"
```

**Method 2: Atomic Write**

```bash
#!/bin/bash
TARGET_SESSION="$1"

# Write to temp file
TMP_FILE=$(mktemp)
cat > "$TMP_FILE" <<EOF
{
  "schema": 1,
  "id": "$TARGET_SESSION",
  "path": "sessions/$TARGET_SESSION"
}
EOF

# Atomic rename
mv "$TMP_FILE" .snek/active.json
```

### Listing Sessions

```bash
#!/bin/bash
for session_dir in .snek/sessions/*/; do
  session_file="$session_dir/session.json"
  if [ -f "$session_file" ]; then
    session_id=$(basename "$session_dir")
    session_name=$(jq -r '.name' "$session_file")
    echo "$session_id: $session_name"
  fi
done
```

### Deleting a Session

**⚠️ Warning**: Check if session is active before deleting!

```bash
#!/bin/bash
SESSION_TO_DELETE="$1"

# Check if active
ACTIVE_ID=$(jq -r '.id' .snek/active.json)
if [ "$ACTIVE_ID" = "$SESSION_TO_DELETE" ]; then
  echo "Error: Cannot delete active session!"
  exit 1
fi

# Delete
rm -rf ".snek/sessions/$SESSION_TO_DELETE"
echo "Deleted session: $SESSION_TO_DELETE"
```

### Cloning a Session

```bash
#!/bin/bash
SOURCE_SESSION="$1"
NEW_SESSION=$(uuidgen | tr '[:upper:]' '[:lower:]')

# Copy directory
cp -r ".snek/sessions/$SOURCE_SESSION" ".snek/sessions/$NEW_SESSION"

# Update session.json with new ID
jq --arg id "$NEW_SESSION" '.id = $id | .version = 0' \
  ".snek/sessions/$NEW_SESSION/session.json" > tmp.json
mv tmp.json ".snek/sessions/$NEW_SESSION/session.json"

echo "Cloned $SOURCE_SESSION to $NEW_SESSION"
```

---

## File Operations

### Atomic Writes

**Always use atomic writes** to prevent corruption during LSP file watching:

```bash
# ❌ WRONG - Direct write
echo '{"schema":1,...}' > .snek/active.json

# ✅ CORRECT - Atomic write
TMP=$(mktemp)
echo '{"schema":1,...}' > "$TMP"
mv "$TMP" .snek/active.json
```

### Adding Chat Messages

```bash
#!/bin/bash
SESSION_ID="$1"
ROLE="$2"
CONTENT="$3"

CHAT_FILE=".snek/sessions/$SESSION_ID/chat.json"

# Read current messages
MESSAGES=$(jq '.messages' "$CHAT_FILE")

# Add new message
NEW_MESSAGES=$(echo "$MESSAGES" | jq --arg role "$ROLE" --arg content "$CONTENT" \
  '. += [{"role": $role, "content": $content}]')

# Write atomically
TMP=$(mktemp)
jq --argjson messages "$NEW_MESSAGES" '.messages = $messages' "$CHAT_FILE" > "$TMP"
mv "$TMP" "$CHAT_FILE"
```

### Adding Code Contexts

```bash
#!/bin/bash
SESSION_ID="$1"
FILE_PATH="$2"
START_LINE="$3"
END_LINE="$4"
LANGUAGE="$5"
DESCRIPTION="$6"

CONTEXT_FILE=".snek/sessions/$SESSION_ID/context.json"

# Get absolute file URI
ABS_PATH=$(realpath "$FILE_PATH")
FILE_URI="file://$ABS_PATH"

# Create context entry (LSP will populate code field)
NEW_CONTEXT=$(jq -n \
  --arg uri "$FILE_URI" \
  --argjson start "$START_LINE" \
  --argjson end "$END_LINE" \
  --arg lang "$LANGUAGE" \
  --arg desc "$DESCRIPTION" \
  '{
    uri: $uri,
    start_line: $start,
    end_line: $end,
    language_id: $lang,
    code: "",
    description: $desc,
    last_modified: ""
  }')

# Add to contexts array
TMP=$(mktemp)
jq --argjson ctx "$NEW_CONTEXT" '.contexts += [$ctx]' "$CONTEXT_FILE" > "$TMP"
mv "$TMP" "$CONTEXT_FILE"

echo "Added context from $FILE_PATH:$START_LINE-$END_LINE"
```

### Removing Code Contexts

```bash
#!/bin/bash
SESSION_ID="$1"
CONTEXT_URI="$2"

CONTEXT_FILE=".snek/sessions/$SESSION_ID/context.json"

# Remove context by URI
TMP=$(mktemp)
jq --arg uri "$CONTEXT_URI" '.contexts |= map(select(.uri != $uri))' \
  "$CONTEXT_FILE" > "$TMP"
mv "$TMP" "$CONTEXT_FILE"

echo "Removed context: $CONTEXT_URI"
```

### Updating Session Name

```bash
#!/bin/bash
SESSION_ID="$1"
NEW_NAME="$2"

SESSION_FILE=".snek/sessions/$SESSION_ID/session.json"

# Update name and version
TMP=$(mktemp)
jq --arg name "$NEW_NAME" \
  '.name = $name | .version += 1 | .updated_at = (now | strftime("%Y-%m-%dT%H:%M:%SZ"))' \
  "$SESSION_FILE" > "$TMP"
mv "$TMP" "$SESSION_FILE"

echo "Updated session name to: $NEW_NAME"
```

---

## Validation Rules

### Schema Validation

All files must have `"schema": 1` field.

### JSON Validation

All files must be valid JSON:
```bash
# Validate JSON
jq empty .snek/active.json
jq empty .snek/sessions/*/session.json
jq empty .snek/sessions/*/chat.json
jq empty .snek/sessions/*/context.json
```

### UUID Validation

```bash
# Validate UUID format (RFC 4122)
UUID_REGEX='^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$'

if [[ "$SESSION_ID" =~ $UUID_REGEX ]]; then
  echo "Valid UUID"
else
  echo "Invalid UUID"
fi
```

### File Existence

Before switching sessions, verify:
```bash
#!/bin/bash
SESSION_ID="$1"
SESSION_DIR=".snek/sessions/$SESSION_ID"

if [ ! -d "$SESSION_DIR" ]; then
  echo "Error: Session directory not found"
  exit 1
fi

if [ ! -f "$SESSION_DIR/session.json" ]; then
  echo "Error: session.json not found"
  exit 1
fi

if [ ! -f "$SESSION_DIR/chat.json" ]; then
  echo "Error: chat.json not found"
  exit 1
fi

if [ ! -f "$SESSION_DIR/context.json" ]; then
  echo "Error: context.json not found"
  exit 1
fi

echo "Session valid: $SESSION_ID"
```

### Chat Message Validation

```bash
# Check message roles
jq -e '.messages | all(.role == "system" or .role == "user" or .role == "assistant")' \
  .snek/sessions/*/chat.json

# Check messages have content
jq -e '.messages | all(.content | type == "string" and length > 0)' \
  .snek/sessions/*/chat.json
```

### Context Validation

```bash
# Check URIs are file:// scheme
jq -e '.contexts | all(.uri | startswith("file://"))' \
  .snek/sessions/*/context.json

# Check line ranges are valid
jq -e '.contexts | all(.end_line > .start_line)' \
  .snek/sessions/*/context.json

# Check language IDs are not empty
jq -e '.contexts | all(.language_id | type == "string" and length > 0)' \
  .snek/sessions/*/context.json
```

---

## Use Cases & Workflows

### Workflow 1: Starting Fresh Project

```bash
# 1. LSP creates .snek/ on first run (automatically)
# 2. Check default session
jq . .snek/active.json

# 3. Add team coding standards to chat
./snek-cli chat add system "Follow PEP 8 for Python code"
./snek-cli chat add system "Use type hints for all functions"

# 4. Start coding with AI completions
```

### Workflow 2: Feature Branch Development

```bash
# 1. Create session for feature branch
./snek-cli session create "Feature: Payment Integration"

# 2. Add relevant context
./snek-cli context add src/models/payment.py 0 50 "Payment model"
./snek-cli context add src/utils/stripe.py 10 60 "Stripe helpers"

# 3. Add architectural notes to chat
./snek-cli chat add user "We use Stripe for payments"
./snek-cli chat add assistant "Understood. I'll suggest Stripe SDK patterns"

# 4. Work on feature with rich context
```

### Workflow 3: Team Standards

```bash
# 1. Create shared session for team
./snek-cli session create "Team: Backend Standards"

# 2. Add comprehensive style guide
./snek-cli chat add system "$(cat docs/style-guide.md)"

# 3. Add example implementations as context
./snek-cli context add examples/good-handler.go 0 100 "Example handler"
./snek-cli context add examples/good-test.go 0 80 "Example test"

# 4. Share session ID with team
./snek-cli session export team-backend > team-backend.snek

# 5. Team members import
./snek-cli session import team-backend.snek
```

### Workflow 4: Learning from AI Conversations

```bash
# 1. Have conversation with ChatGPT about project architecture
# 2. Export conversation to JSON
# 3. Convert to chat.json format
./snek-cli chat import chatgpt-export.json

# 4. Future completions use architectural decisions from conversation
```

### Workflow 5: Multi-Project Setup

```bash
# Project 1: Python Backend
cd ~/projects/api-backend
export SNEK_API_KEY="..."
# LSP creates .snek/ here

# Project 2: React Frontend
cd ~/projects/web-frontend
export SNEK_API_KEY="..."
# LSP creates separate .snek/ here

# Each project has independent sessions and contexts
```

---

## CLI Tool Requirements

### Core Commands

#### Session Management

```bash
# List all sessions
snek-cli session list
# Output: <uuid>: <name> [ACTIVE]

# Create new session
snek-cli session create <name>
# Output: Created session: <uuid>

# Switch session
snek-cli session switch <uuid>
# Output: Switched to session: <uuid>

# Delete session
snek-cli session delete <uuid>
# Output: Deleted session: <uuid>

# Rename session
snek-cli session rename <uuid> <new-name>
# Output: Renamed session to: <new-name>

# Show session details
snek-cli session show [uuid]
# Output: Full session info (defaults to active)

# Clone session
snek-cli session clone <source-uuid> [new-name]
# Output: Cloned to: <new-uuid>

# Export session
snek-cli session export [uuid] > session.snek
# Output: JSON export

# Import session
snek-cli session import session.snek
# Output: Imported as: <uuid>
```

#### Chat Management

```bash
# List messages
snek-cli chat list [session-uuid]
# Output: Numbered list of messages

# Add message
snek-cli chat add <role> <content> [session-uuid]
# Output: Added message #N

# Edit message
snek-cli chat edit <index> <new-content> [session-uuid]
# Output: Updated message #N

# Delete message
snek-cli chat delete <index> [session-uuid]
# Output: Deleted message #N

# Clear all messages
snek-cli chat clear [session-uuid]
# Output: Cleared all messages

# Import from file
snek-cli chat import <file.json> [session-uuid]
# Output: Imported N messages

# Export to file
snek-cli chat export [session-uuid] > chat-export.json
# Output: JSON export
```

#### Context Management

```bash
# List contexts
snek-cli context list [session-uuid]
# Output: Numbered list with URIs and descriptions

# Add context
snek-cli context add <file> <start-line> <end-line> [description] [session-uuid]
# Output: Added context from <file>

# Remove context
snek-cli context remove <index> [session-uuid]
# Output: Removed context #N

# Clear all contexts
snek-cli context clear [session-uuid]
# Output: Cleared all contexts

# Show context details
snek-cli context show <index> [session-uuid]
# Output: Full context info including code

# Update context range
snek-cli context update <index> <start-line> <end-line> [session-uuid]
# Output: Updated context #N
```

#### Utility Commands

```bash
# Initialize .snek in current directory
snek-cli init
# Output: Initialized .snek/

# Validate all files
snek-cli validate
# Output: Validation results

# Show statistics
snek-cli stats
# Output: Number of sessions, messages, contexts, etc.

# Cleanup old/orphaned files
snek-cli cleanup
# Output: Removed N orphaned files
```

### Output Formats

Support multiple output formats:

```bash
# Human-readable (default)
snek-cli session list

# JSON
snek-cli session list --json

# Table
snek-cli session list --table

# CSV
snek-cli context list --csv
```

### Interactive Mode

```bash
# Launch interactive TUI
snek-cli interactive

# Features:
# - Browse sessions
# - Edit chat messages
# - Manage contexts
# - Switch sessions
# - Live validation
```

### Configuration

```bash
# CLI config file: ~/.config/snek/config.toml
[default]
api_key = "sk-..."
api_base_url = "https://api.openai.com/v1"
default_max_tokens = 1600

[editor]
preferred = "vim"  # For editing messages

[display]
default_format = "table"
colors = true
```

---

## Security Considerations

### Sensitive Data

**⚠️ Warning**: `.snek/` may contain sensitive information:
- API keys in environment (not stored in files)
- Proprietary code in `context.json`
- Internal architecture details in `chat.json`
- Business logic in chat history

### Recommendations

1. **Add to .gitignore**:
   ```gitignore
   .snek/
   ```

2. **Encrypt for sharing**:
   ```bash
   tar czf - .snek/ | openssl enc -aes-256-cbc -out snek-backup.tar.gz.enc
   ```

3. **Audit before export**:
   ```bash
   # Review all contexts
   jq '.contexts[].code' .snek/sessions/*/context.json
   
   # Review all chat messages
   jq '.messages[].content' .snek/sessions/*/chat.json
   ```

4. **Use separate sessions**:
   - Personal session: Your preferences
   - Team session: Shared standards only
   - Project session: Project-specific context

### File Permissions

```bash
# Recommended permissions
chmod 700 .snek/
chmod 600 .snek/*.json
chmod 700 .snek/sessions/
chmod 600 .snek/sessions/*/*.json
```

---

## Migration & Versioning

### Schema Evolution

Current schema version: **1**

When schema changes (future):

1. LSP reads `"schema"` field
2. If version < current, migrate on load
3. Write migrated data with new schema version
4. Keep backward compatibility for 1 major version

### Example Migration (Future)

```bash
# Migrate schema 1 → schema 2
for file in .snek/sessions/*/session.json; do
  TMP=$(mktemp)
  jq '.schema = 2 | .new_field = "default"' "$file" > "$TMP"
  mv "$TMP" "$file"
done
```

### Backup Before Migration

```bash
#!/bin/bash
BACKUP_DIR="$HOME/.snek-backups/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP_DIR"
cp -r .snek/ "$BACKUP_DIR/"
echo "Backed up to: $BACKUP_DIR"
```

---

## Troubleshooting

### Common Issues

**1. Session Not Found**
```bash
# Check session exists
ls .snek/sessions/

# Verify active.json points to valid session
jq '.id' .snek/active.json
```

**2. Invalid JSON**
```bash
# Validate all JSON files
find .snek -name "*.json" -exec sh -c 'jq empty "$1" || echo "Invalid: $1"' _ {} \;
```

**3. Context Code Not Updating**
```bash
# Check if LSP is running
ps aux | grep snek

# Check if file URI is valid
jq '.contexts[].uri' .snek/sessions/*/context.json

# Verify file exists
jq -r '.contexts[].uri' .snek/sessions/*/context.json | \
  sed 's|file://||' | \
  xargs -I {} sh -c 'test -f "{}" || echo "Missing: {}"'
```

**4. LSP Not Detecting Changes**
```bash
# Check file watcher
# Look for [SNEK] messages in LSP stderr

# Try touching the file to trigger watch
touch .snek/active.json
```

---

## Summary

### Key Points

✅ **`.snek/` is the configuration hub**
- All session data in one place
- Automatically created by LSP
- Safe to delete and reinitialize

✅ **Sessions are isolated**
- Each session = separate context
- Switch sessions to change AI behavior
- Clone sessions for experimentation

✅ **Chat history provides context**
- Import conversations from AI tools
- Add team coding standards
- Document architectural decisions

✅ **Code contexts are auto-managed**
- You specify file + line range
- LSP extracts and updates code
- Always in sync with source files

✅ **All operations are atomic**
- Use tmp + rename pattern
- Safe for concurrent access
- LSP detects changes instantly

✅ **CLI tool is essential**
- Manage sessions easily
- Edit chat history
- Add/remove contexts
- Validate configuration

---

## Next Steps

1. **Build CLI Tool**: Use specifications in this guide
2. **Test Workflows**: Try use cases with real projects
3. **Create Templates**: Build session templates for common scenarios
4. **Share Sessions**: Export/import for team collaboration
5. **Automate**: Script common operations for efficiency

---

**End of .snek Folder Guide**

For technical integration details, see `EXTENSION_INTEGRATION_GUIDE.md`.

