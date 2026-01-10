# Change: Add CSR creation and CSR-based issuance

## Why
Some users already have endpoint-owned keys and CSRs, and the current flow requires manual DNS name entry. Supporting CSR import and CSR creation improves issuance flexibility while keeping secrets local.

## What Changes
- Add a CSR-based issuance path that accepts a CSR file and derives SANs from the CSR instead of manual name entry.
- Add CSR creation that generates a key in the Rust core and writes a CSR file for later issuance.
- Validate CSRs in the Rust core (signature, SANs, key type) before starting issuance.
- Store CSR metadata on issued certificates (subject, SANs, key type, source).

## Impact
- Affected specs: certificate-issuance
- Affected code: `src/` Issue flow UI, `src-tauri/` core commands/issuance, DTOs
