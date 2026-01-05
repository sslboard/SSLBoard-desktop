# Change: Automate Certificate Issuance Workflow

## Why
Currently, the UI is responsible for polling DNS propagation and triggering the final issuance step. This leads to a brittle experience and requires the UI to stay open and active. Moving this logic to the backend enables a "fire and forget" experience for managed issuance, with the backend handling the lifecycle asynchronously.

## What Changes
- **MODIFIED**: `start_managed_issuance` now initiates an asynchronous background task in the Rust core.
- **ADDED**: Automatic backend polling for DNS-01 challenge propagation.
- **ADDED**: Automatic ACME finalization and certificate storage upon successful propagation.
- **ADDED**: Manual intervention requirement only for "Manual" DNS providers or when automatic validation fails.
- **ADDED**: Background task status reporting via Tauri events to keep the UI informed without manual polling.
- **UI**: REFACTOR "Issue" page to be a passive observer of the background task state.
- **UI**: REMOVE "Check Propagation" and "Complete Issuance" buttons for automated providers.
- **UI**: ADD persistent background task monitoring to the app shell (sidebar/topbar).

## Impact
- Affected specs: `certificate-issuance`, `rust-code-quality`
- Affected code: `src-tauri/src/issuance/`, `src/hooks/useManagedIssuanceFlow.ts`

