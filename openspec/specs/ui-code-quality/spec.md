# ui-code-quality Specification

## Purpose
TBD - created by archiving change refactor-ui-code-quality. Update Purpose after archive.
## Requirements
### Requirement: UI file size limits enforced
UI source files under `src/` SHALL remain within reasonable size limits to preserve readability and single responsibility.

#### Scenario: Oversized UI module is split
- **WHEN** a UI module grows beyond a reasonable size threshold (e.g., large pages, components, hooks)
- **THEN** it SHALL be split into smaller, focused modules
- **AND** each resulting module SHALL have a clear responsibility and name

### Requirement: UI logic separation of concerns
The UI SHALL separate responsibilities across pages, components, hooks, and pure helper modules.

#### Scenario: Page orchestrates, hook owns state
- **WHEN** a workflow requires multi-step state and async orchestration (e.g., issuance)
- **THEN** the workflow state machine SHALL live in a dedicated hook
- **AND** the page SHALL primarily compose presentational sections using that hook’s view-model

#### Scenario: Pure helpers live in src/lib
- **WHEN** logic is pure (validation, formatting, mapping) and does not require React state
- **THEN** it SHALL live in `src/lib/` (or a focused subfolder)

### Requirement: Consistent UI naming conventions
The UI codebase SHALL use consistent naming conventions for handlers, props, and boolean state.

#### Scenario: Event handler naming is consistent
- **WHEN** a component receives event handler props
- **THEN** handler props SHALL be named `onX` (e.g., `onSubmit`, `onCancel`, `onSelectIssuer`)
- **AND** local handler functions SHALL be named `handleX`

#### Scenario: Boolean state naming is consistent
- **WHEN** state represents a boolean condition
- **THEN** it SHALL be named with `isX` / `hasX` / `canX` prefixes

### Requirement: Import organization is standardized
UI modules SHALL keep imports organized and stable to minimize diff noise.

#### Scenario: Imports are grouped
- **WHEN** a UI module has both external and internal imports
- **THEN** external imports SHALL appear first
- **AND** a blank line SHALL separate external and internal imports

### Requirement: Consistent UI error handling patterns
User-facing error messages in the UI SHALL be produced via consistent normalization patterns.

#### Scenario: Async failures show normalized errors
- **WHEN** an async UI action fails (IPC, validation, I/O)
- **THEN** the UI SHALL normalize the error into a user-facing message
- **AND** the error message SHALL avoid leaking secrets or raw sensitive content

### Requirement: Dead code removed
Unused exports, unused branches, and unreachable UI code paths SHALL be removed.

#### Scenario: Unused code is eliminated during refactor
- **WHEN** TypeScript strict checks or refactors reveal unused exports or unreachable code
- **THEN** that code SHALL be removed
- **AND** `npm run build` SHALL succeed

