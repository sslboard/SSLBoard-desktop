# Implementation Tasks for Rust Code Quality Refactoring

## 1. Critical File Splitting

- [x] 1.1 Create new module `src-tauri/src/core/commands/dns_provider_validation.rs`
  - Move validation functions: `validate_cloudflare_token`, `validate_digitalocean_token`, `validate_route53_token`
  - Extract `categorize_dns_error` function
  - Add proper error types and return `Result<TokenValidationResult, ValidationError>`

- [x] 1.2 Create new module `src-tauri/src/core/commands/dns_provider_creation.rs`
  - Move `dns_provider_create` function
  - Extract helper functions: `provider_type_to_string`, `create_route53_credentials`, `create_api_token_credential`
  - Simplify the large match statement into smaller focused functions

- [x] 1.3 Create new module `src-tauri/src/core/commands/dns_provider_management.rs`
  - Move `dns_provider_update`, `dns_provider_delete`, `dns_provider_list`, `dns_provider_resolve` functions
  - Extract `provider_record_to_dto` conversion function
  - Add consistent error handling patterns

- [x] 1.4 Create new module `src-tauri/src/core/commands/dns_provider_testing.rs`
  - Move `dns_provider_test` function (the 127-line complex function)
  - Break down into smaller functions: `execute_test`, `poll_propagation`, `cleanup_test_records`
  - Add proper timeout handling and error categorization

- [x] 1.5 Refactor main `dns_providers.rs` to import and expose the new modules
  - Keep only the high-level command exports
  - Ensure all Tauri command attributes are preserved
  - Update all import statements in lib.rs

## 2. Error Handling Improvements

- [x] 2.1 Remove silent error handling in `dns_providers.rs:155`
  - Replace `let _ = secrets.delete_secret(secret_ref);` with proper error handling
  - Add logging for cleanup failures
  - Return appropriate errors to the UI

- [x] 2.2 Remove silent error handling in `legacy/acme_issuer.rs`
  - Replace `let _ = self.secrets.resolve_secret(&existing_ref)?;` with proper handling
  - Add migration error logging and recovery strategies

- [x] 2.3 Fix unsafe error handling in `dns.rs:351`
  - Replace `ManualDnsAdapter::query_txt(&record_name).expect("dns query should succeed")` with proper error handling
  - Return meaningful errors for DNS resolution failures

- [x] 2.4 Fix unsafe error handling in `lib.rs:71`
  - Replace `.expect("error while running tauri application")` with proper error logging
  - Consider graceful shutdown on initialization failures

## 3. Import and Dead Code Cleanup

- [x] 3.1 Remove unused import in `src-tauri/src/core/types.rs`
  - Remove `PropagationState` from line 5
  - Verify it's not used elsewhere in the file

- [x] 3.2 Audit for other unused imports
  - Run `cargo check` and fix any warnings
  - Remove unused dependencies if found

## 4. Naming and Code Consistency

- [x] 4.1 Standardize variable naming in command handlers
  - Replace generic `req` parameters with descriptive names like `create_req`, `test_req`
  - Use consistent naming for database connections (`conn` vs `store`)

- [x] 4.2 Improve function naming consistency
  - Standardize conversion functions (`to_dto` vs `record_to_dto`)
  - Use consistent prefixes for similar operations

- [x] 4.3 Fix ambiguous variable names
  - Replace `adapter` with more specific names (`dns_adapter`, `provider_adapter`)
  - Clarify `result` variables with context-specific names

## 5. Logging Framework Implementation

- [x] 5.1 Replace `eprintln!` in `secrets/manager.rs`
  - Implement structured logging with `tracing` or `log` crate
  - Remove sensitive data from log messages
  - Add appropriate log levels (debug/info/warn/error)

- [x] 5.2 Standardize logging across DNS modules
  - Replace `eprintln!` in `issuance/dns.rs`
  - Add consistent logging for DNS resolution failures
  - Include relevant context in log messages

- [x] 5.3 Add logging configuration
  - Set up log level filtering
  - Configure output format for development vs production

## 6. Code Duplication Removal

- [x] 6.1 Extract common HTTP client patterns
  - Create `HttpClient` abstraction for DNS providers
  - Implement connection pooling
  - Standardize timeout and retry logic

- [x] 6.2 Extract error handling patterns
  - Create `handle_api_error` function for common HTTP error responses
  - Standardize rate limit and authentication error handling
  - Reduce duplicated error message strings

- [x] 6.3 Extract validation helpers
  - Create `validate_domain_suffixes` function
  - Standardize label validation across providers
  - Add consistent input sanitization

## 7. Performance and Security Improvements

- [x] 7.1 Implement connection pooling for DNS APIs
  - Add `reqwest` client reuse across requests
  - Configure appropriate connection limits
  - Add connection timeout handling

- [x] 7.2 Remove sensitive data from debug logs
  - Audit all logging statements for potential secret exposure
  - Replace sensitive data with redacted placeholders
  - Add secure logging utilities

- [x] 7.3 Improve timeout handling
  - Add configurable timeouts for all external API calls
  - Implement proper cancellation for long-running operations
  - Add timeout error categorization

## 8. Testing and Validation

- [x] 8.1 Update existing tests to work with refactored code
  - Fix import paths in test modules
  - Update function signatures if changed
  - Ensure test coverage remains intact

- [x] 8.2 Add tests for error handling improvements
  - Test proper error propagation
  - Verify logging behavior
  - Test timeout and cancellation scenarios

- [x] 8.3 Validate refactoring with `cargo check` and `cargo test`
  - Ensure no compilation errors
  - Verify all Tauri commands still work
  - Run integration tests for DNS providers
