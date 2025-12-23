## 1. Implementation

- [x] 1.1 Add Rust distribution module for export (`src-tauri/src/distribution/export.rs`) that can write PEM files safely (no secrets to UI).
- [x] 1.2 Define IPC DTOs for export options and results in Rust core commands; expose a `export_certificate_pem` Tauri command.
- [x] 1.3 Implement key-availability checks:
  - If certificate has no `managed_key_ref`, “include key” is rejected (and UI disables it).
- [x] 1.4 Implement best-effort file permissions and safe write semantics with explicit overwrite confirmation (clear error messaging).
- [x] 1.5 Add UI “Export…” action for Managed certificates with an options modal:
  - bundle selection (cert/chain/fullchain); emits the standard set of PEM files
  - include private key (when available; default off)
  - destination selection (directory) + suggested subfolder name (derived from primary DNS name; editable)
- [x] 1.6 Add user-facing warnings/confirmations before exporting private keys.
- [x] 1.7 Add user-facing prompt before overwriting existing files.

## 2. Tests

- [x] 2.1 Add Rust unit tests for PEM writing and safe path handling (temporary directory).
- [x] 2.2 Add a small integration test for the export command (happy path without private key; private key path when a test secret ref exists).
