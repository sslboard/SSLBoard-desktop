# Change: Add ACME profile support for issuance

## Why
Let’s Encrypt and other ACME CAs can advertise issuance profiles that constrain certificate properties. Supporting profiles lets users choose the right issuance class while keeping validation inside the Rust core.

## What Changes
- Surface ACME profiles advertised by the CA and allow optional selection during issuance.
- Validate profile selection in the Rust core and include it in the ACME newOrder request when set.
- Persist the selected profile in certificate metadata for display and filtering.

## Impact
- Affected specs: certificate-issuance
- Affected code: Rust ACME client, IPC DTOs, Issue workflow UI
