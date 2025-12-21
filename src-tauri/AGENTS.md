# Rust Core Notes (src-tauri/)

## Scope
- Rust core owns secrets, issuance, storage, and distribution logic.
- The UI should only call into Tauri commands for sensitive work.

## Structure
- `src/core/`: IPC commands, DTOs, error types.
- `src/secrets/`: Secret storage adapters and vault logic.
- `src/issuance/`: ACME, DNS-01, and private PKI logic.
- `src/storage/`: Non-secret metadata storage (inventory, issuers, DNS configs).

## Conventions
- Add new Tauri commands under `src/core/commands` and register them in `src/lib.rs`.
- Prefer explicit error types and `Result` returns for command handlers.
- Avoid blocking the main thread; use async where appropriate.
