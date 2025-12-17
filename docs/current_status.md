# Current Status (Desktop)

- Secret storage abstraction implemented in Rust with OS keyring backing (`src-tauri/src/secrets/*`); prefixed refs (`sec_`), non-secret metadata persisted in SQLite, and Tauri commands for list/create/update/delete wired into the app.
- Settings → Secrets UI added (`src/pages/Settings.tsx`) to list refs and handle add/replace/remove flows; UI only sends secret bytes into Rust once and operates on metadata afterward.
- OpenSpec change `add-secret-store-abstraction` validated with all tasks checked; spec reflects prefixed refs, fail-fast on missing OS store, metadata in DB, and replace-with-same-ref semantics.
- `cargo fmt` + `cargo check` pass; current warnings are expected unused helpers (`resolve_secret`, `SecretStore::retrieve`, `InventoryStore::insert_certificate`) pending integration with issuance/DNS flows.
- Inventory foundations are in place (`src-tauri/src/storage/inventory.rs` + Tauri commands) with a Certificates page that lists metadata only; a demo seed record is inserted in debug builds and via `seed_fake_certificate`.
- Issuance/DNS flows are not implemented yet; OpenSpec changes `add-acme-issuer-sandbox` and `add-dns-challenge-engine-manual-adapter` remain at 0/5 tasks.
- Key dependencies added: `uuid` (prefixed ref ids) and `keyring` (OS store). Secrets DB lives at `app_data_dir()/secrets.sqlite`; key material stays in OS keychain (fail fast if unavailable, no file fallback).
- Placeholders / pending items:
  - Certificates list can seed a fake record via UI and `seed_fake_certificate`; real issuance not wired yet.
  - Discover page is still placeholder content.
  - Unused helper warnings remain until issuance/DNS call sites use `resolve_secret`, `SecretStore::retrieve`, and `insert_certificate`.
- Next logical work: hook `resolve_secret` into DNS/ACME adapters, persist/use secret refs in provider configs, flesh out issuer + DNS adapters per pending changes, and optionally quiet unused warnings once call sites exist.
