## 1. Dependencies & Setup
- [ ] 1.1 Add crate dependencies: `aes-gcm`, `rand`, `base64`, `zeroize`
- [ ] 1.2 Update `Cargo.lock` with `cargo build`

## 2. Schema Migration
- [ ] 2.1 Add `ciphertext BLOB` column to `secret_metadata` table schema
- [ ] 2.2 Implement migration logic in `SecretMetadataStore::init_schema`
- [ ] 2.3 Enforce `0600` file permissions on `secrets.sqlite` creation/migration

## 3. Master Key Management
- [ ] 3.1 Create `master_key.rs` module with get/create logic for keyring storage
- [ ] 3.2 Implement master key generation (32 random bytes, base64 encoded for keyring)
- [ ] 3.3 Implement master key retrieval from keyring

## 4. Encrypted Secret Store
- [ ] 4.1 Create `EncryptedSecretStore` struct implementing `SecretStore` trait
- [ ] 4.2 Implement AES-256-GCM encryption (random nonce, prepended to ciphertext)
- [ ] 4.3 Implement AES-256-GCM decryption (extract nonce, decrypt)
- [ ] 4.4 Add `store_ciphertext` and `get_ciphertext` methods to `SecretMetadataStore`

## 5. Lock/Unlock API
- [ ] 5.1 Define `SecretVault` trait with `is_unlocked`, `unlock`, `lock` methods
- [ ] 5.2 Implement vault state management with `RwLock<Option<SecretVec<u8>>>` for cached master key
- [ ] 5.3 Implement `lock()` with explicit zeroization of cached key
- [ ] 5.4 Implement `unlock()` to retrieve master key from keyring and cache

## 6. Migration Logic
- [ ] 6.1 Detect if migration is needed (no `ciphertext` column or null ciphertext rows)
- [ ] 6.2 For each secret: attempt old keyring read, encrypt, store, delete old entry
- [ ] 6.3 For missing keyring entries: delete metadata row (orphan cleanup, log at debug)
- [ ] 6.4 Mark migration complete (pragma or flag)

## 7. Integration
- [ ] 7.1 Refactor `SecretManager` to use `EncryptedSecretStore` + vault
- [ ] 7.2 Expose `lock` and `unlock` as Tauri commands
- [ ] 7.3 Update `resolve_secret` to check vault unlock state

## 8. Documentation
- [ ] 8.1 Update `docs/db.md` with new `ciphertext` column in `secret_metadata`
- [ ] 8.2 Document lock/unlock behavior in code comments

## 9. Cleanup
- [ ] 9.1 Remove or deprecate `KeyringSecretStore` per-secret methods (keep master key logic)
- [ ] 9.2 Remove old keyring entries after successful migration verification

