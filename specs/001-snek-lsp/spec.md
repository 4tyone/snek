# Feature Specification: Snek Language Server Protocol

**Feature Branch**: `001-snek-lsp`  
**Created**: 2025-11-02  
**Status**: Draft  
**Input**: User description: "Snek Language Server Protocol implementation with AI-powered code completions, chat history integration, and code context management"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic AI Code Completion (Priority: P1)

As a developer, I want to receive AI-powered code suggestions as I type, so that I can write code faster and with fewer errors.

**Why this priority**: This is the core value proposition of the LSP. Without inline completions, the system provides no immediate user value. This story delivers the fundamental capability that all other features build upon.

**Independent Test**: Can be fully tested by opening a code file in an editor with the LSP running, typing code, and verifying that relevant completion suggestions appear. Delivers immediate value by accelerating code writing.

**Acceptance Scenarios**:

1. **Given** the LSP is running and connected to an editor, **When** a developer types code and pauses at a cursor position, **Then** the system provides a relevant code completion suggestion within 2 seconds
2. **Given** a developer is writing a function, **When** they position the cursor mid-function, **Then** the completion respects the existing code context (prefix and suffix)
3. **Given** the API endpoint is unavailable, **When** a completion is requested, **Then** the system logs an error and returns gracefully without crashing

---

### User Story 2 - Team Style Guidance via Chat History (Priority: P2)

As a team lead, I want to configure coding conventions and style preferences that influence all code suggestions, so that the AI helps enforce team standards automatically.

**Why this priority**: While completions work without this, team consistency is a major pain point in collaborative development. This story enables the LSP to become a style guide enforcer, significantly increasing its value in team settings.

**Independent Test**: Can be tested by adding team style messages to the chat history (e.g., "use snake_case for functions"), requesting completions, and verifying that suggestions follow the specified conventions. Delivers value by reducing code review cycles.

**Acceptance Scenarios**:

1. **Given** chat history contains "We use snake_case for functions", **When** a developer requests a completion for a new function, **Then** the suggested code uses snake_case naming
2. **Given** chat history specifies "Avoid async_trait", **When** a completion involves traits, **Then** the suggestion does not include async_trait patterns
3. **Given** multiple style preferences in chat history, **When** completions are generated, **Then** all applicable preferences are respected simultaneously

---

### User Story 3 - Code Context from Related Files (Priority: P3)

As a developer, I want the AI to reference code from other files in my project (like models, utilities, or types), so that suggestions are aware of my existing codebase structure.

**Why this priority**: This significantly improves completion quality by making the AI aware of project-specific patterns, but the system is still valuable without it. This story transforms the LSP from a generic code assistant to a project-aware intelligent helper.

**Independent Test**: Can be tested by adding a file reference to context.json (e.g., models.rs with a User struct), requesting a completion that could use that struct, and verifying the suggestion correctly references the User type. Delivers value by reducing context switching between files.

**Acceptance Scenarios**:

1. **Given** context.json references a User struct from models.rs, **When** a developer writes code that could use User, **Then** the completion correctly references and instantiates the User struct
2. **Given** a context file is modified, **When** the file is saved, **Then** the LSP automatically updates its internal context within 250 milliseconds without requiring a restart
3. **Given** context.json lists 5 different code snippets, **When** a completion is requested, **Then** only relevant contexts appear in the prompt (not all 5 unnecessarily)

---

### User Story 4 - Session Management (Priority: P3)

As a developer, I want to switch between different coding sessions with different chat histories and contexts, so that I can maintain separate AI configurations for different projects or features.

**Why this priority**: This is a convenience feature that enables advanced workflows but isn't required for basic functionality. It delivers value for developers working on multiple projects or feature branches simultaneously.

**Independent Test**: Can be tested by creating two sessions with different chat histories, switching between them, and verifying that completions reflect the active session's configuration. Delivers value by eliminating configuration conflicts between projects.

**Acceptance Scenarios**:

1. **Given** two sessions exist (one for frontend, one for backend), **When** a developer switches the active session, **Then** subsequent completions use the new session's chat history and context within 300 milliseconds
2. **Given** a session is created with a specific name, **When** the developer lists available sessions, **Then** the session appears with its name and metadata
3. **Given** multiple sessions exist, **When** the LSP starts, **Then** it loads the last active session automatically

---

### Edge Cases

- What happens when the context.json references a file that doesn't exist or has been deleted?
- How does the system handle extremely large context files (>10,000 lines)?
- What happens when the API key is invalid or expired?
- How does the system behave when network connectivity is lost mid-completion?
- What happens if chat.json or context.json contains malformed JSON?
- How does the system handle concurrent file modifications (e.g., git operations changing multiple files)?
- What happens when the cursor is positioned in an invalid location (beyond file end)?
- How does the system handle special characters or non-UTF8 content in files?
- What happens when session.json version counter overflows or becomes corrupted?
- How does the system behave when disk space is exhausted during file writes?

## Requirements *(mandatory)*

### Functional Requirements

#### Core Completion Engine

- **FR-001**: System MUST provide inline code completion suggestions based on cursor position, surrounding code (prefix and suffix), and language context
- **FR-002**: System MUST return completion suggestions within 2 seconds of request
- **FR-003**: System MUST gracefully handle API failures by logging errors and returning empty completions without crashing
- **FR-004**: System MUST support OpenAI-compatible API endpoints with configurable base URL and API key
- **FR-005**: System MUST extract code prefix (all text before cursor) and suffix (all text after cursor) from the active document

#### Chat History Integration

- **FR-006**: System MUST maintain a chat history file (chat.json) containing messages with roles (system, user, assistant) and content
- **FR-007**: System MUST include all chat history messages in the completion prompt, preserving order
- **FR-008**: System MUST allow external tools (CLI/extension) to add, modify, or clear chat messages via file operations
- **FR-009**: System MUST reload chat history within 250 milliseconds when chat.json is modified
- **FR-010**: System MUST apply chat history guidance (coding conventions, style preferences) to all completion suggestions

#### Code Context Management

- **FR-011**: System MUST maintain a context.json file listing code snippets from other files with URI, line ranges, language, code content, and optional descriptions
- **FR-012**: System MUST watch all files referenced in context.json for changes
- **FR-013**: System MUST incrementally update context when a watched file changes (without full session reload)
- **FR-014**: System MUST append code contexts to the final completion prompt after chat history
- **FR-015**: System MUST extract specified line ranges from context files and keep them synchronized with file modifications
- **FR-016**: System MUST update context within 250 milliseconds when a watched context file is modified

#### Session Management

- **FR-017**: System MUST support multiple sessions, each with independent chat history and context
- **FR-018**: System MUST maintain an active.json file indicating which session is currently active
- **FR-019**: System MUST switch sessions within 300 milliseconds when active.json is modified
- **FR-020**: System MUST persist session metadata including session ID, name, version counter, token limits, and last updated timestamp
- **FR-021**: System MUST increment session version counter after any modification to session files

#### File System Integration

- **FR-022**: System MUST create .snek directory structure if it doesn't exist on startup
- **FR-023**: System MUST initialize default session with empty chat and context on first run
- **FR-024**: System MUST use atomic file operations (write to temp file, then rename) for all file modifications
- **FR-025**: System MUST debounce file change notifications with 200 millisecond delay to coalesce rapid changes
- **FR-026**: System MUST maintain all session state in RAM for zero disk I/O during completion requests

#### Document Tracking

- **FR-027**: System MUST track the currently active document's content via LSP didOpen, didChange, and didClose notifications
- **FR-028**: System MUST support full document synchronization (entire document sent on each change)
- **FR-029**: System MUST extract prefix and suffix relative to cursor position from the tracked document

#### Error Handling & Resilience

- **FR-030**: System MUST log all errors with context information without exposing sensitive data (API keys)
- **FR-031**: System MUST continue operating when individual context files fail to load or parse
- **FR-032**: System MUST validate JSON schema before processing session files
- **FR-033**: System MUST provide default empty values when optional files (chat.json, context.json) are missing
- **FR-034**: System MUST handle file permission errors gracefully with informative error messages

### Key Entities

- **Session**: Represents a coding context with unique ID, name, version counter, token limits, chat history, code contexts, and last updated timestamp. Multiple sessions can exist but only one is active at a time.

- **Chat Message**: Represents a single message in the chat history with role (system/user/assistant) and content. Messages are ordered and influence completion style and conventions.

- **Code Context**: Represents a code snippet from another file with URI, start line, end line, language ID, code content, optional description, and last modified timestamp. Contexts are watched for changes and kept synchronized.

- **Context Snapshot**: In-memory representation of the active session including session ID, version, limits, chat messages, and code contexts. Used for fast completion generation without disk I/O.

- **Document Content**: Represents the currently active file being edited with URI, full text content, and language ID. Used to extract prefix and suffix for completions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers receive code completion suggestions within 2 seconds of cursor pause 95% of the time
- **SC-002**: System handles 100 completion requests per minute without performance degradation
- **SC-003**: Chat history modifications are reflected in subsequent completions within 250 milliseconds
- **SC-004**: Context file changes are detected and synchronized within 250 milliseconds
- **SC-005**: Session switches complete within 300 milliseconds with new session context active
- **SC-006**: System operates with memory footprint under 10MB for typical sessions (10 chat messages, 5 context files)
- **SC-007**: System continues providing completions even when 20% of context files are unavailable or malformed
- **SC-008**: 90% of completion requests succeed when API is available (accounting for network transients)
- **SC-009**: System initializes and becomes ready for completions within 1 second of editor startup
- **SC-010**: Zero data loss during concurrent file modifications (atomic writes prevent corruption)
- **SC-011**: Developers can add team coding conventions via chat history and see them applied in completions within one completion cycle
- **SC-012**: Code contexts from related files appear in completion prompts with correct line ranges and content

## Assumptions

- Developers have a stable internet connection for API calls (offline mode is not in scope)
- The OpenAI-compatible API endpoint supports the standard chat completions format
- Editors/extensions handle the LSP stdio protocol correctly
- File system supports atomic rename operations
- Session files remain under 1MB each (reasonable for typical usage)
- Context files are text-based and UTF-8 encoded
- Developers have read/write permissions for the .snek directory
- The workspace root can be determined by walking up the directory tree
- API keys are provided via environment variables (secure storage is future work)
- Single developer per workspace (no concurrent multi-user editing)

## Out of Scope

- Streaming completions (tokens returned as generated) - future enhancement
- Local model inference (llama.cpp, etc.) - future enhancement
- Automatic context detection based on imports/symbols - future enhancement
- Multi-document tracking (only active document is tracked)
- Diagnostics and linting features
- Code actions and refactoring suggestions
- Telemetry and analytics
- Configuration UI (configuration is file-based only)
- Authentication beyond API key
- Rate limiting and quota management
- Offline mode or caching of completions
- Multi-language support in a single session (one language per completion)
- Real-time collaboration features
- Version control integration
- Workspace-wide symbol indexing
