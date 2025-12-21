# Change: Issuer management as first-class entities

## Why
Issuer configuration is currently handled as settings with fixed entries. We need a first-class issuer entity so users can add new issuers with issuer-specific parameters while keeping safety defaults for Let's Encrypt staging and production.

## What Changes
- Introduce issuer entities stored in the issuance metadata store with issuer-specific parameters and lifecycle state (selected/disabled).
- Seed Let's Encrypt staging and production issuers as first-class entries, with staging selected by default.
- Require contact email and explicit terms acceptance before registering ACME issuers (and before enabling production issuers).
- Update UI workflows to create, edit, disable, and select issuers (separate from raw settings).

## Impact
- Affected specs: `issuer-management`
- Affected code: `src-tauri/src/storage/issuer.rs`, `src-tauri/src/core/commands.rs`, `src-tauri/src/core/types.rs`, `src/pages/Settings.tsx` (or new issuer UI)
- Data: `issuance.sqlite` schema and migrations
