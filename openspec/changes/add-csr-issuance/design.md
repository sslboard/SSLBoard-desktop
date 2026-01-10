## Context
CSR import and CSR creation introduce new issuance paths and touch key handling. The UI remains untrusted, and secrets must stay in the Rust core.

## Goals / Non-Goals
- Goals: support CSR import for issuance, support CSR generation with managed keys, and keep CSR validation and key material in the core.
- Non-Goals: exporting private keys to the UI or building a general CSR editor for arbitrary extensions.

## Decisions
- Decision: Parse and validate CSR PEM in the Rust core (signature, SANs, key algorithm) before issuance.
- Decision: CSR-based issuance derives identifiers from the CSR; the UI does not send DNS names.
- Decision: CSR generation creates and stores a managed private key in the Rust core and writes the CSR PEM to a user-selected path.
- Decision: Accept CN-only CSRs with a warning; SANs are preferred and validated when present.
- Decision: Only RSA and ECDSA CSRs are supported, matching the managed key options.
- Decision: Favor a dedicated CSR issuance command for clarity unless the existing issuance command remains readable with a new CSR option.
- Decision: Use the Tauri dialog file path and have the Rust core read CSR bytes from disk to keep the UI untrusted and avoid moving file contents across IPC.

## Risks / Trade-offs
- CSR validation gaps could allow malformed requests; mitigate with strict parsing and explicit error surfaces.
- CSR generation with managed keys shifts key ownership to the app; mitigate by making the CSR creation flow explicit and opt-in.

## Migration Plan
- No data migration required; new metadata fields are additive.

## Open Questions
- Should CSR generation allow optional export of the private key as a separate user action?
