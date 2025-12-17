# Current Status (Desktop)

- Secret storage abstraction implemented in Rust with OS keyring backing (`src-tauri/src/secrets/*`); prefixed refs (`sec_`), non-secret metadata persisted in SQLite, and Tauri commands for list/create/update/delete wired into the app.
- Settings → Secrets UI added (`src/pages/Settings.tsx`) to list refs and handle add/replace/remove flows; UI only sends secret bytes into Rust once and operates on metadata afterward.
- OpenSpec change `add-secret-store-abstraction` validated with all tasks checked; spec reflects prefixed refs, fail-fast on missing OS store, metadata in DB, and replace-with-same-ref semantics.
- `cargo fmt` + `cargo check` pass; current warnings are expected unused helpers (`resolve_secret`, `SecretStore::retrieve`, `InventoryStore::insert_certificate`) pending integration with issuance/DNS flows.
- Inventory foundations are in place (`src-tauri/src/storage/inventory.rs` + Tauri commands) with a Certificates page that lists metadata only; a demo seed record is inserted in debug builds and via `seed_fake_certificate`.
- Issuance/DNS flows are partially in place: DNS-01 manual adapter + propagation polling implemented (ureq DoH lookup every 2s, 90s budget) with UI stepper on `Issue` page; OpenSpec change `add-dns-challenge-engine-manual-adapter` tasks are complete; ACME issuer sandbox change still pending.
- New OpenSpec change `add-issue-certificate-flow` drafted for step 5 (wizard, key-gen vs CSR import, DNS-01 gating, Managed persistence).
- Key dependencies added: `uuid` (prefixed ref ids) and `keyring` (OS store). Secrets DB lives at `app_data_dir()/secrets.sqlite`; key material stays in OS keychain (fail fast if unavailable, no file fallback).
- Placeholders / pending items:
  - Certificates list can seed a fake record via UI and `seed_fake_certificate`; real issuance not wired yet.
  - Discover page is still placeholder content.
  - Unused helper warnings remain until issuance/DNS call sites use `resolve_secret`, `SecretStore::retrieve`, and `insert_certificate`.
- ACME issuer scaffolding added: issuer store (`issuance.sqlite`) with staging/prod rows, Tauri commands to list/select issuers and ensure an ACME account, UI issuer settings with sandbox banner, contact email, and account key generate/import flows; secret kind validation guards prevent mixing DNS and ACME refs. Deadlock in issuer store fixed; account key auto-regenerates if missing.
- Next logical work: implement `add-issue-certificate-flow` (ACME order orchestration, key-gen/CSR paths, Managed certificate persistence), wire actual ACME registration/order flows, hook `resolve_secret` into DNS/ACME adapters, persist/use secret refs in provider configs (beyond manual adapter defaults), flesh out issuer + DNS adapters, and optionally quiet unused warnings once call sites exist.
