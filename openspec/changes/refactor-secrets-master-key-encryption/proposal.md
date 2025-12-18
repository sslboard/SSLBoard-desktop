# Change: Refactor Secrets to Master Key Encryption

## Why
The current per-secret keyring approach requires the OS to potentially unlock the keyring for each secret access, degrading UX—especially on macOS where Keychain prompts can be aggressive. A single master key stored in the keyring, with secrets encrypted in SQLite, limits unlock prompts to once per session and gives explicit lock/unlock control to the user.

## What Changes
- **BREAKING**: Migrate from per-secret keyring entries to a single master key in the keyring
- Secrets are now encrypted with AES-256-GCM and stored in `secrets.sqlite` (new `ciphertext` column)
- Master key is generated on first use (32 bytes, random) and stored in the OS keyring
- Add explicit `lock()` / `unlock()` functions for session-based master key management (allows a "lock" UI control)
- Master key is zeroized from memory when locked
- Database files created/migrated to strict `0600` permissions (owner read/write only)
- Migration: silently remove secret metadata rows if their corresponding keyring entry is missing (orphaned secrets cleaned up)

## Impact
- Affected specs: `secret-store`
- Affected code:
  - `src-tauri/src/secrets/manager.rs` — refactor to use encrypted store
  - `src-tauri/src/secrets/keyring_store.rs` — repurpose for master key only
  - `src-tauri/src/secrets/metadata.rs` — add `ciphertext BLOB` column + migration
  - `src-tauri/src/secrets/store.rs` — new `EncryptedSecretStore` implementation
  - `docs/db.md` — update schema documentation
- Dependencies to add: `aes-gcm`, `rand`, `base64`, `zeroize`

