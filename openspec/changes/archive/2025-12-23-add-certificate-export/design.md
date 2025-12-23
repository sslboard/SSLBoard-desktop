## Context

Certificate export is a distribution workflow that can easily become a security footgun, especially when exporting private keys. This project treats the UI as untrusted and keeps secrets in the Rust core and OS secret store.

## Goals / Non-Goals

- Goals:
  - Provide a minimal, safe **PEM export** workflow for Managed certificates.
  - Preserve the trust boundary: no private key material over IPC to the UI.
  - Guard rails for private key export (explicit user intent + warnings).
  - Best-effort restrictive file permissions on written artifacts.
- Non-Goals:
  - PFX/PKCS#12 export (future).
  - Automated distribution to endpoints, GitOps, Kubernetes, etc. (separate capabilities).
  - Strong user-presence gating (e.g., Touch ID) for export (future; see `add-macos-biometric-keychain`).

## Decisions

- Decision: **Export runs in Rust core** and writes files to disk.
  - The UI provides only: certificate id, export bundle selection, include-key flag, and a user-selected destination (directory or base path).
  - The Rust core resolves and loads:
    - Certificate PEM chain from local storage/inventory.
    - Private key (only if `managed_key_ref` exists) from `SecretStore`.
- Decision: **Output directory + app-created subfolder**.
  - The user selects an output directory.
  - The UI proposes a default subfolder name derived from the certificate’s first DNS SAN, and the user can edit it.
  - If multiple SANs exist, the default folder name MAY include a suffix (e.g., `example.com+2`) to reflect additional names.
- Decision: **Explicit confirmation for private key export**.
  - Exporting `privkey.pem` requires a clear warning and a second confirmation click (UI-level affordance), but the Rust core still enforces the include-key flag and key-availability checks.
- Decision: **Best-effort file protections**.
  - The Rust core SHALL attempt to create files with restrictive permissions (e.g., `0600` on Unix) and avoid overwriting unless explicitly requested.
  - Overwrite behavior: the system prompts before overwriting existing files and proceeds only when the user confirms.

## Risks / Trade-offs

- Risk: Users export private keys to insecure locations (cloud-synced folders, world-readable paths).
  - Mitigation: strong warning copy, confirmation, and safe defaults (key export unchecked).
- Risk: UI can be compromised and attempt to export keys silently.
  - Mitigation: export requires explicit user action in the UI and should remain a high-friction workflow; future work may add OS user-presence gating in Rust.
- Trade-off: Cross-platform file permission semantics differ.
  - Mitigation: best-effort approach; document behavior and warn if protections cannot be applied.

## Migration Plan

No migration required; this adds a new workflow and a new Tauri command.

## Open Questions

- Should the folder default be normalized/shortened when SAN lists are long (beyond a simple `+N` suffix)?


