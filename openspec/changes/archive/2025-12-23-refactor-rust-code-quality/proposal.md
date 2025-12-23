# Change: Refactor Rust Code Quality

## Why

The Rust codebase currently has several code quality issues that impact maintainability, performance, and security. A comprehensive code review identified critical issues including overly complex files (672 lines), silent error handling, unused code, and poor naming conventions. These issues need to be addressed to ensure the codebase remains maintainable as the project grows.

## What Changes

### Critical Fixes
- **Split oversized `dns_providers.rs` file** (672 lines) into smaller, focused modules
- **Remove silent error handling** - replace `let _ =` patterns with proper error handling
- **Remove unused imports** - clean up `PropagationState` import in `types.rs`
- **Replace `unwrap()`/`expect()`** in production code with proper error handling

### Code Organization
- **Extract validation logic** into dedicated modules
- **Separate concerns** in command handlers vs utility functions
- **Standardize error handling patterns** across DNS provider implementations

### Quality Improvements
- **Implement proper logging framework** - replace `eprintln!` with structured logging
- **Improve naming consistency** - standardize variable and function names
- **Reduce code duplication** - extract common HTTP client patterns
- **Add connection pooling** for DNS provider API clients

### Security Enhancements
- **Remove debug logging of sensitive data** from production code
- **Standardize error messages** to avoid information leakage
- **Implement consistent timeout handling** for external API calls

## Impact

- **Affected specs**: rust-code-quality (new capability)
- **Affected code**:
  - `src-tauri/src/core/commands/dns_providers.rs` - major refactoring
  - `src-tauri/src/core/types.rs` - remove unused import
  - `src-tauri/src/issuance/dns_providers/` - standardize error handling
  - `src-tauri/src/secrets/manager.rs` - replace debug logging
  - `src-tauri/src/issuance/dns.rs` - fix unsafe error handling
- **Breaking changes**: None - this is internal refactoring only
- **Performance impact**: Improved - connection pooling and better error handling
- **Security impact**: Improved - removes information leakage vectors
