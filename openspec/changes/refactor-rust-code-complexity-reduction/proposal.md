# Change: Refactor Rust code complexity reduction

## Why

The Rust codebase contains several large files (400+ lines) that violate the established code quality standards and contain mixed responsibilities. Specifically, `src-tauri/src/issuance/dns_providers/base.rs` (603 lines), `src-tauri/src/issuance/flow.rs` (570 lines), and `src-tauri/src/core/commands/dns_provider_testing.rs` (286 lines) are overly complex and difficult to maintain. These files mix DNS provider abstractions with testing logic, ACME workflow orchestration with low-level operations, and repetitive validation patterns. Refactoring these files will improve code maintainability, testability, and separation of concerns while reducing complexity.

## What Changes

- **Extract DNS testing logic** from `base.rs` into dedicated `dns_testing.rs` module
- **Break down massive issuance functions** in `flow.rs` into focused workflow components
- **Extract repetitive DNS provider validation** logic into shared validation module
- **Create new modules** for DNS cleanup and ACME workflow orchestration
- **Update rust-code-quality spec** to include new requirements for module organization
- **No breaking API changes** - all public interfaces remain compatible

## Impact

- Affected specs: rust-code-quality
- Affected code: `src-tauri/src/issuance/dns_providers/base.rs`, `src-tauri/src/issuance/flow.rs`, `src-tauri/src/core/commands/dns_provider_testing.rs`, and related modules
- **BREAKING**: Internal module structure changes - no external API impact
