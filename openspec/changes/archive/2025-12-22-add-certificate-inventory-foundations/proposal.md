# Change: Certificate inventory foundations (local metadata store + read APIs)

## Why
The app needs a durable inventory of certificates (including external/discovered ones) before any issuance workflows can be reliable, testable, or auditable.

## What Changes
- Add a local, non-secret metadata store for certificate inventory records.
- Add read APIs for inventory: `list_certificates` and `get_certificate`.
- Add a basic UI screen to view the certificate list and a details panel with an empty-state.

## Impact
- Affected specs: `certificate-inventory`
- Affected code (planned): `src-tauri/src/storage/`, `src-tauri/src/core/commands.rs`, `src-tauri/src/core/types.rs`, `src/` inventory UI routes/components


