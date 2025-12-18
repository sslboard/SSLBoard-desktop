## Context
The current secrets implementation stores each secret as a separate OS keyring entry. While secure, this causes UX friction:
- macOS Keychain may prompt for unlock on each secret access
- Multiple secrets = multiple prompts (especially after sleep/lock)
- No explicit user control over "locking" secrets

This change moves to a master-key architecture where:
- A single 256-bit master key is stored in the OS keyring
- All secrets are encrypted with AES-256-GCM and stored in SQLite
- The master key is cached in memory after unlock and zeroized on lock

## Goals / Non-Goals
- **Goals**:
  - Single keyring unlock per session
  - Explicit lock/unlock API for UI "lock vault" button
  - Strong authenticated encryption (AES-256-GCM)
  - Secure master key handling (zeroize on lock)
  - Strict file permissions on secrets database (0600)
  - Clean migration from per-secret keyring entries
- **Non-Goals**:
  - User-provided master password / key derivation (use OS keyring as-is)
  - Cloud sync or backup encryption key management
  - Hardware security module / Secure Enclave integration (future enhancement)

## Decisions

### Cipher: AES-256-GCM
- **Why**: Industry-standard authenticated encryption, widely audited, fast on modern CPUs (AES-NI)
- **Alternatives considered**:
  - ChaCha20-Poly1305: Good alternative, slightly faster on non-AES-NI hardware; AES-GCM chosen for broader familiarity
  - XChaCha20-Poly1305: Larger nonce reduces collision risk, but 96-bit nonce with random generation is safe for our scale

### Nonce strategy: Random 96-bit per encryption
- Each encrypt operation generates a fresh random 12-byte nonce
- Nonce prepended to ciphertext for storage: `nonce (12 bytes) || ciphertext || tag`
- At our secret volume (<10k secrets lifetime), birthday collision risk is negligible (2^48 encryptions needed)

### Master key storage format
- Keyring entry: service=`sslboard-desktop`, user=`master_key`
- Value: base64-encoded 32-byte key
- Backward-compatible: if no master key exists, generate one

### Memory handling: Zeroize
- Master key cached in `SecretVec<u8>` (from `zeroize` crate) or manual zeroize on drop
- `lock()` explicitly zeroizes and clears cached key
- `unlock()` retrieves from keyring and caches

### Lock/Unlock API
```rust
pub trait SecretVault: Send + Sync {
    fn is_unlocked(&self) -> bool;
    fn unlock(&self) -> Result<(), SecretError>;
    fn lock(&self);
}
```
- On startup, vault is locked
- Any operation requiring secrets calls `unlock()` (or returns error if locked)
- UI can call `lock()` to wipe master key from memory

### Database file permissions
- `secrets.sqlite` created with mode `0600` (owner read/write only)
- On migration, `chmod 0600` applied if current permissions are too permissive
- Platform note: Windows NTFS doesn't use Unix permissions; rely on user-profile ACLs

### Migration strategy
1. On startup, check if `ciphertext` column exists in `secret_metadata`
2. If not, run schema migration to add `ciphertext BLOB`
3. Generate or retrieve master key from keyring
4. For each row in `secret_metadata`:
   - Attempt to read secret from old keyring entry
   - If found: encrypt with master key, store in `ciphertext`, delete old keyring entry
   - If not found: delete the metadata row (orphaned secret cleanup)
5. Mark migration complete (e.g., user_version pragma or migration table)

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Master key loss = all secrets lost | Document backup strategy; master key is in OS keyring which users should back up |
| Memory exposure of master key | Use `zeroize` crate; lock() explicitly wipes; avoid debug logging |
| DB file permission bypass on Windows | Accept limitation; Windows uses ACLs, not chmod |
| Migration deletes orphaned secrets silently | Log at debug level for diagnostics; this is cleanup of already-broken state |
| AES-GCM nonce reuse catastrophe | Random nonces with 96-bit space; at <10k secrets, collision is astronomically unlikely |

## Migration Plan

### Phase 1: Schema + Code (this change)
1. Add `ciphertext BLOB` column to `secret_metadata`
2. Implement `EncryptedSecretStore` with AES-256-GCM
3. Implement `SecretVault` trait (lock/unlock/is_unlocked)
4. Add migration logic (keyring → DB encryption)
5. Add file permission enforcement

### Phase 2: UI integration (future change)
- Add lock/unlock button to UI shell
- Show locked state indicator
- Auto-lock on app idle (optional, configurable)

### Rollback
- Keep old `KeyringSecretStore` code until migration is proven stable
- If issues arise, secrets are still in keyring (until cleanup phase completes)
- Rollback window: until old keyring entries are deleted

## Open Questions
- Should we support re-keying (master key rotation)?
  - Defer to future change; not required for initial implementation
- Should lock happen automatically on app minimize/idle?
  - Defer to UI integration phase

