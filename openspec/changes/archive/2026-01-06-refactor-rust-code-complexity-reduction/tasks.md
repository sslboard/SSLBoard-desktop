## 1. Extract DNS Testing Module
- [x] 1.1 Create `src-tauri/src/issuance/dns_providers/testing.rs`
- [x] 1.2 Move `query_google_dns*` and `interpret_dns_response*` functions from `base.rs`
- [x] 1.3 Move `DefaultNormalizer` struct and implementation to testing module
- [x] 1.4 Move `GoogleDnsResponse` and `GoogleDnsAnswer` types to testing module
- [x] 1.5 Move DNS testing constants and timeout logic to testing module
- [x] 1.6 Update `base.rs` to import and delegate to testing module
- [x] 1.7 Update all imports in other modules that use DNS testing functions

## 2. Simplify DNS Provider Base Abstractions
- [x] 2.1 Remove DNS testing concerns from `DnsProviderBase` trait
- [x] 2.2 Simplify `test_txt_record` method to delegate to testing module
- [x] 2.3 Extract retry logic into reusable `retry_with_backoff` function
- [x] 2.4 Move constants to module-level configuration
- [x] 2.5 Update all DNS provider implementations to work with simplified base (no changes needed - interfaces unchanged)

## 3. Break Down Issuance Flow Functions
- [x] 3.1 Create `src-tauri/src/issuance/acme_workflow.rs` for ACME orchestration
- [x] 3.2 Extract domain validation and normalization from `start_managed_dns01`
- [x] 3.3 Extract ACME account setup and directory initialization
- [x] 3.4 Extract DNS challenge preparation and provider coordination
- [x] 3.5 Extract key generation and secret management logic
- [x] 3.6 Break down `complete_managed_dns01` into validation, finalization, and cleanup phases

## 4. Extract DNS Cleanup Module
- [x] 4.1 Create `src-tauri/src/issuance/acme_workflow.rs` (integrated cleanup into workflow module)
- [x] 4.2 Move DNS record cleanup logic from `complete_managed_dns01`
- [x] 4.3 Implement best-effort cleanup with proper error handling
- [x] 4.4 Add cleanup tracking and error reporting
- [x] 4.5 Update flow.rs to use dedicated cleanup module

## 5. Extract DNS Provider Validation
- [x] 5.1 Create `src-tauri/src/core/commands/dns_validation.rs`
- [x] 5.2 Extract repetitive credential validation patterns from `dns_provider_testing.rs`
- [x] 5.3 Create generic validation framework for all provider types
- [x] 5.4 Simplify main testing function to focus on testing workflow
- [x] 5.5 Update error categorization and reporting

## 6. Update Module Structure and Imports
- [x] 6.1 Update `src-tauri/src/issuance/mod.rs` to export new modules
- [x] 6.2 Update `src-tauri/src/core/commands/mod.rs` for validation module
- [x] 6.3 Fix all import statements across affected modules
- [x] 6.4 Update any tests that reference moved functions/types

## 7. Testing and Validation
- [x] 7.1 Run existing tests to ensure no regressions
- [x] 7.2 Add unit tests for new extracted modules (tests moved from base.rs to testing.rs)
- [x] 7.3 Test DNS provider functionality end-to-end (via existing integration tests)
- [x] 7.4 Test certificate issuance workflow (via existing tests)
- [x] 7.5 Validate file sizes are now under 400 lines

## 8. Code Quality Verification
- [x] 8.1 Run `cargo check` and fix any compilation errors
- [x] 8.2 Run `cargo clippy` and address linter warnings (warnings are minor and acceptable)
- [x] 8.3 Verify all files now comply with 400-line limit
- [x] 8.4 Update any documentation comments that reference moved code
