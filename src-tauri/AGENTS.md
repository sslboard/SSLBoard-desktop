# Rust Core Notes (src-tauri/)

## Scope
- Rust core owns secrets, issuance, storage, and distribution logic.
- The UI should only call into Tauri commands for sensitive work.

## Structure
- `src/core/`: IPC commands, DTOs, error types.
- `src/secrets/`: Secret storage adapters and vault logic.
  - `biometric_store.rs`: macOS biometric-protected keychain storage (Touch ID/Face ID).
  - `keyring_store.rs`: Cross-platform OS keyring storage.
  - `mod.rs`: Platform detection and master key store factory.
- `src/issuance/`: ACME, DNS-01, and private PKI logic.
- `src/storage/`: Non-secret metadata storage (inventory, issuers, DNS configs).

## Conventions
- Add new Tauri commands under `src/core/commands` and register them in `src/lib.rs`.
- Prefer explicit error types and `Result` returns for command handlers.
- Avoid blocking the main thread; use async where appropriate.

## Biometric Authentication
- On macOS, the system automatically uses Touch ID/Face ID for master key access when biometric hardware is available.
- The `BiometricKeyringStore` creates Keychain items with `SecAccessControl` using `kSecAccessControlBiometryAny` flags.
- Biometric prompts appear automatically when accessing protected secrets - no explicit unlock UI needed.
- Falls back to standard Keychain authentication (passcode) if biometrics unavailable or fail.
