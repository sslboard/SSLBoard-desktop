# Change: Add key algorithm options for issuance

## Why
Users need the ability to issue certificates with ECC keys and stronger RSA key sizes for compliance and performance trade-offs. The current managed issuance flow only supports RSA-2048.

## What Changes
- Add key algorithm parameters to managed issuance requests (RSA 2048/3072/4096, ECDSA P-256/P-384).
- Update the Issue page to let users select the desired key algorithm and size/curve.
- Validate requested key parameters in the Rust core before issuance and use them during key generation.
- Maintain a backward-compatible default when the UI does not provide key parameters.
- Persist the selected key algorithm/size/curve in certificate metadata for display and filtering.

## Impact
- Affected specs: `certificate-issuance` (new capability spec)
- Affected code: `src/lib/issuance.ts`, `src/hooks/useManagedIssuanceFlow.ts`, `src/components/issue/*`, `src-tauri/src/core/types.rs`, `src-tauri/src/issuance/*`, `src-tauri/src/storage/*`
