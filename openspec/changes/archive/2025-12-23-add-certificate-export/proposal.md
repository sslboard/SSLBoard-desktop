# Change: Add certificate export for managed certificates (PEM)

## Why

Users need a safe, explicit way to export managed certificates (and optionally their private keys) for use in servers, proxies, and other tooling. Export is a core “distribution” workflow and must preserve the project’s trust boundary: the UI remains untrusted and MUST NOT handle raw private key material.

## What Changes

- Add an “Export…” action for **Managed** certificates in the UI.
- Support exporting **PEM** outputs:
  - Certificate (`cert.pem`)
  - Chain (`chain.pem`)
  - Full chain (`fullchain.pem`)
  - Optional private key (`privkey.pem`) when the app has custody of the key.
- Implement export in the **trusted Rust core**; the UI passes only a certificate id + export options and receives only success/failure and output paths.
- Add guard rails:
  - Explicit warning/confirmation before exporting private keys.
  - Disable “include private key” when the certificate has no managed key reference (e.g., CSR-imported).
  - Best-effort restrictive file permissions (e.g., `0600` on Unix-like systems).
  - Prompt before overwriting existing files.
  - Organize exports under a per-certificate subfolder (defaulted from the certificate’s primary DNS name, editable by the user).

## Impact

- Affected specs: `certificate-export` (new capability)
- Affected code (expected):
  - Rust: `src-tauri/src/distribution/export.rs` (new), `src-tauri/src/core/commands/*` (new command + DTOs), `src-tauri/src/lib.rs` (command registration)
  - UI: `src/components/certificates/CertificateDetail.tsx` (Export action), `src/lib/certificates.ts` (IPC wrapper), UI modal/components for export options


