# Change: Issue certificate flow (generate key or CSR)

## Why
- Deliver end-to-end issuance so users can obtain a real sandbox certificate with DNS-01 validation.
- Support both managed-key and CSR-import paths while keeping secrets in the Rust core.

## What Changes
- Add a multi-step issuance wizard (domains → key mode → DNS-01 → finalize) that surfaces DNS instructions and success state.
- Implement Rust orchestration to start ACME orders, handle manual DNS propagation checks, and finalize/download the chain.
- Support both key-generation (SecretStore-backed) and CSR-import flows, persisting issued certificate metadata as Managed.

## Impact
- Affected specs: certificate-issuance
- Affected code: `src-tauri/src/issuance/*`, `src-tauri/src/storage/*`, `src/lib/*`, `src/pages/Issue.tsx`
