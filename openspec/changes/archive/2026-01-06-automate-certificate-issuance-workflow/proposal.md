# Change: Automate Certificate Issuance Workflow

## Why
Currently, the UI is responsible for polling DNS propagation and triggering the final issuance step. This leads to a brittle experience and requires the UI to stay open and active. Moving this logic to the backend enables a "fire and forget" experience for managed issuance, with the backend handling the lifecycle asynchronously.

## What Changes
- **MODIFIED**: The Rust core handles DNS propagation polling and ACME finalization during `complete_managed_issuance`, so the UI does not need standalone DNS polling commands.
- **ADDED**: The Issue page auto-advances for automated DNS providers by calling `complete_managed_issuance` after `start_managed_issuance`.
- **ADDED**: Manual intervention is required only when the returned DNS records indicate a `manual` adapter; the UI provides a user confirmation to proceed.

## Impact
- Affected specs: `certificate-issuance`, `rust-code-quality`
- Affected code: `src-tauri/src/issuance/`, `src/hooks/useManagedIssuanceFlow.ts`
