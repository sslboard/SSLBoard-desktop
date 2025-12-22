# Change: Secret storage abstraction (OS-backed, reference-only to UI)

## Why
Issuance and DNS automation require secrets (ACME account keys, private keys, DNS API tokens). To preserve the trust boundary, the UI must never see raw secrets and all secret custody must remain in the local Rust core.

## What Changes
- Add `secrets::store::SecretStore` abstraction in Rust, backed by the OS secret store.
- Store and retrieve secrets by **reference ids**, never returning secret bytes to the UI.
- Add minimal settings UI to manage secret references (create/list/remove) without displaying raw secret values.

## Impact
- Affected specs: `secret-store`
- Affected code (planned): `src-tauri/src/secrets/`, `src-tauri/src/core/commands.rs`, `src-tauri/src/core/types.rs`, `src/` settings UI


