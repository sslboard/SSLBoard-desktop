# Change: Improve Rust Error Handling and Diagnostics

## Why
Several Rust core paths currently suppress errors or fall back silently, which makes failures hard to diagnose and can hide data corruption or missing secrets. We need consistent, user-understandable errors without panics or silent drops.

## What Changes
- Ensure parsing failures for persisted DNS provider data (JSON and timestamps) are logged with context and surfaced or explicitly fallback with a warning.
- Ensure best-effort cleanup/rollback failures are logged (metadata deletes, secret resolution checks).
- Ensure config parsing for provider zone overrides logs failures instead of returning `None` silently.
- Preserve non-crashing behavior while improving error clarity and traceability.

## Impact
- Affected specs: `specs/rust-code-quality/spec.md`
- Affected code:
  - `src-tauri/src/storage/dns.rs`
  - `src-tauri/src/secrets/manager.rs`
  - `src-tauri/src/issuance/flow.rs`
  - `src-tauri/src/core/commands/dns_challenge.rs`
  - `src-tauri/src/core/commands/dns_provider_management.rs`
