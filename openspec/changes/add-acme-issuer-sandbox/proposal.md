# Change: Issuer interface + Let’s Encrypt sandbox (ACME staging) issuer

## Why
Issuance must be pluggable from day 1, and the default experience must be safe. Using Let’s Encrypt staging as the default issuer enables end-to-end issuance testing without risking production rate limits or accidental real certificates.

## What Changes
- Define an `Issuer` interface in the Rust core for account management, ordering, challenge retrieval, finalization, and certificate download.
- Implement an ACME issuer configured to Let’s Encrypt **staging** endpoint by default.
- Persist `issuer_id` and issuer configuration in the local metadata store, and persist the ACME account key as a secret reference.
- Add UI issuer settings with a clear “Sandbox” indicator.

## Impact
- Affected specs: `issuer-acme`
- Affected code (planned): `src-tauri/src/issuance/`, `src-tauri/src/core/commands.rs`, `src-tauri/src/core/types.rs`, `src/` settings UI


