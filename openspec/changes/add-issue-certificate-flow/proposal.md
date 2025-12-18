# Change: Issue certificate flow (generate key or CSR)

## Why
- Deliver end-to-end issuance so users can obtain a real sandbox certificate with DNS-01 validation.
- Support managed-key issuance while keeping secrets in the Rust core. CSR import will follow later.

## What Changes
- Add a multi-step issuance wizard (domains → DNS-01 → finalize) that surfaces DNS instructions and success state.
- Implement Rust orchestration to start ACME orders, handle manual DNS propagation checks, and finalize/download the chain.
- Support managed key-generation (SecretStore-backed), persisting issued certificate metadata as Managed.

## Impact
- Affected specs: certificate-issuance
- Affected code: `src-tauri/src/issuance/*`, `src-tauri/src/storage/*`, `src/lib/*`, `src/pages/Issue.tsx`
