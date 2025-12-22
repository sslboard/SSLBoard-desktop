## ADDED Requirements

### Requirement: File size limits enforced
Source code files SHALL NOT exceed 400 lines. Files exceeding this limit MUST be split into smaller, focused modules with single responsibilities.

#### Scenario: Large command file is split
- **WHEN** a command module exceeds 400 lines
- **THEN** the file SHALL be split into logical sub-modules
- **AND** each sub-module SHALL have a clear, single responsibility
- **AND** the main module SHALL import and re-export functionality

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
All external API calls SHALL have proper timeout handling and cancellation support.

#### Scenario: DNS propagation polling respects timeouts
- **WHEN** polling for DNS propagation
- **THEN** timeouts SHALL be configurable
- **AND** operations SHALL be cancellable
- **AND** timeout errors SHALL be properly categorized

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
