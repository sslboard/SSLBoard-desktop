# rust-code-quality Specification

## Purpose
TBD - created by archiving change refactor-rust-code-quality. Update Purpose after archive.
## Requirements
### Requirement: File size limits enforced
Source code files SHALL NOT exceed 400 lines. Files exceeding this limit MUST be split into smaller, focused modules with single responsibilities. Complex functions SHALL be broken down into smaller, focused functions with clear purposes. Mixed responsibilities SHALL be separated into dedicated modules.

#### Scenario: Large command file is split
- **WHEN** a command module exceeds 400 lines
- **THEN** the file SHALL be split into logical sub-modules
- **AND** each sub-module SHALL have a clear, single responsibility
- **AND** the main module SHALL import and re-export functionality

#### Scenario: Complex functions are decomposed
- **WHEN** a function exceeds 50 lines or handles multiple concerns
- **THEN** it SHALL be broken down into smaller, focused functions
- **AND** each function SHALL have a single, clear purpose
- **AND** complex workflows SHALL be orchestrated through dedicated modules

#### Scenario: Mixed responsibilities are separated
- **WHEN** a module contains both high-level orchestration and low-level operations
- **THEN** low-level operations SHALL be extracted to dedicated modules
- **AND** the main module SHALL focus on coordination and error handling
- **AND** clear interfaces SHALL be defined between modules

### Requirement: Silent error handling prohibited
The codebase SHALL NOT use silent error handling patterns. All errors SHALL be properly handled, logged, or propagated.

#### Scenario: Silent errors are replaced with proper handling
- **WHEN** code contains `let _ =` error suppression
- **THEN** it SHALL be replaced with explicit error handling
- **AND** errors SHALL be logged with appropriate context
- **OR** errors SHALL be propagated to callers

#### Scenario: Unsafe unwrap/expect usage eliminated
- **WHEN** code uses `unwrap()` or `expect()` in production paths
- **THEN** it SHALL be replaced with proper error handling
- **AND** meaningful error messages SHALL be provided

#### Scenario: Persisted data parsing failures are surfaced
- **WHEN** parsing stored JSON fields or timestamps fails
- **THEN** the error SHALL be logged with record identifiers and field names
- **AND** the failure SHALL be propagated to callers or a documented fallback SHALL be applied
- **AND** fallback behavior SHALL include a warning log entry

#### Scenario: Best-effort cleanup failures are visible
- **WHEN** a best-effort cleanup or rollback operation fails
- **THEN** the error SHALL be logged with relevant identifiers
- **AND** the primary operation SHALL continue unless the failure is safety-critical

#### Scenario: Config decoding errors are not silent
- **WHEN** parsing configuration blobs (e.g., provider overrides) fails
- **THEN** the error SHALL be logged with provider identifiers
- **AND** the system SHALL fall back to safe defaults without panicking

### Requirement: Consistent error handling patterns
Error handling SHALL follow consistent patterns across the codebase using the `anyhow` crate and custom error types where appropriate.

#### Scenario: DNS provider errors use consistent patterns
- **WHEN** implementing DNS provider API calls
- **THEN** errors SHALL be categorized (Auth, RateLimit, Network, etc.)
- **AND** error messages SHALL follow consistent formatting
- **AND** HTTP status codes SHALL be mapped to appropriate error categories

### Requirement: Proper logging framework implemented
The codebase SHALL use structured logging instead of `eprintln!` for debug and informational messages.

#### Scenario: Debug logging replaced with structured logging
- **WHEN** code uses `eprintln!` for logging
- **THEN** it SHALL be replaced with appropriate logging macros
- **AND** sensitive information SHALL NOT be logged
- **AND** log levels SHALL be appropriately set (debug/info/warn/error)

### Requirement: Naming conventions standardized
Variable and function names SHALL follow consistent conventions and avoid generic names.

#### Scenario: Generic variable names improved
- **WHEN** variables use generic names like `req`, `result`, `adapter`
- **THEN** they SHALL be replaced with descriptive names
- **AND** names SHALL clearly indicate purpose and context

### Requirement: Code duplication eliminated
Common patterns SHALL be extracted into reusable functions to reduce duplication.

#### Scenario: HTTP client patterns abstracted
- **WHEN** multiple modules create HTTP clients with similar configuration
- **THEN** a common HTTP client abstraction SHALL be created
- **AND** connection pooling SHALL be implemented
- **AND** timeout handling SHALL be standardized

#### Scenario: Validation logic consolidated
- **WHEN** similar validation logic exists in multiple places
- **THEN** validation functions SHALL be extracted to shared modules
- **AND** consistent validation error messages SHALL be used

### Requirement: Security logging practices enforced
Logging SHALL NOT expose sensitive information and SHALL follow security best practices.

#### Scenario: Sensitive data redacted in logs
- **WHEN** logging potentially sensitive information
- **THEN** sensitive values SHALL be redacted or omitted
- **AND** only non-sensitive metadata SHALL be logged
- **AND** log messages SHALL be reviewed for information leakage

### Requirement: Connection pooling implemented
HTTP clients SHALL use connection pooling for performance and resource management.

#### Scenario: DNS provider APIs use connection pooling
- **WHEN** making HTTP requests to external DNS APIs
- **THEN** a shared HTTP client with connection pooling SHALL be used
- **AND** appropriate connection limits SHALL be configured
- **AND** connection timeouts SHALL be properly set

### Requirement: Timeout handling standardized
All external API calls and polling tasks SHALL have proper timeout handling.

#### Scenario: DNS propagation polling respects timeouts
- **WHEN** the backend is polling for DNS propagation during issuance completion
- **THEN** the polling SHALL use a bounded timeout and interval
- **AND** timeout errors SHALL be properly categorized and reported to the UI

### Requirement: Dead code removed
Unused imports, functions, and code paths SHALL be removed from the codebase.

#### Scenario: Unused imports cleaned up
- **WHEN** imports are unused according to the compiler
- **THEN** they SHALL be removed
- **AND** `cargo check` SHALL pass without warnings

#### Scenario: Orphaned code paths removed
- **WHEN** code is unreachable or unused
- **THEN** it SHALL be removed
- **OR** it SHALL be properly tested and documented

### Requirement: Module organization standards
Modules SHALL be organized by functional responsibility with clear separation between high-level orchestration, low-level operations, and cross-cutting concerns. DNS testing logic SHALL be separated from DNS provider abstractions. ACME workflow orchestration SHALL be separated from certificate issuance details. Validation logic SHALL be centralized and reusable.

#### Scenario: DNS concerns properly separated
- **WHEN** implementing DNS provider functionality
- **THEN** DNS provider abstractions SHALL be separate from DNS testing logic
- **AND** DNS cleanup operations SHALL be in dedicated modules
- **AND** DNS validation SHALL be centralized and reusable

#### Scenario: Issuance workflow modularized
- **WHEN** implementing certificate issuance workflows
- **THEN** ACME orchestration SHALL be separate from DNS operations
- **AND** domain validation SHALL be in dedicated functions
- **AND** key generation SHALL be abstracted from workflow logic

#### Scenario: Validation logic centralized
- **WHEN** implementing provider or configuration validation
- **THEN** validation patterns SHALL be extracted to reusable functions
- **AND** error categorization SHALL be consistent across providers
- **AND** validation errors SHALL follow standard formats

