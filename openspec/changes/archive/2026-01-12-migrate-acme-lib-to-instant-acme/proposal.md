# Change: Migrate ACME client to instant-acme

## Why
The current acme-lib dependency is limiting and harder to extend. Switching to instant-acme positions the core for future ACME enhancements while keeping current issuance behavior intact.

## What Changes
- Replace the ACME client library with instant-acme in the Rust core.
- Update issuance flow internals to use the instant-acme API while preserving existing behavior.

## Impact
- Affected specs: certificate-issuance
- Affected code: `src-tauri/src/issuance/` ACME workflow, issuer config DTOs, UI Settings and Issue flows
- No user-facing behavior changes expected
