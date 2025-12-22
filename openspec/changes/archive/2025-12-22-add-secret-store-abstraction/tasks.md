## 1. Implementation
- [x] 1.1 Create `src-tauri/src/secrets/` module with `secrets::store::SecretStore` trait and supporting types (secret kind, reference id).
- [x] 1.2 Implement OS-backed secret storage adapter (Keychain / Credential Manager / Secret Service), selecting implementation by platform.
- [x] 1.3 Define secret types to support v0 workflows:
  - ACME account key references
  - Managed private key references
  - DNS provider credential references
- [x] 1.4 Add Rust commands to create/list/remove secret references and to associate references with higher-level records (without exposing secret material).
- [x] 1.5 Add UI “Settings → Secrets” page showing only secret references and allowing add/remove flows driven by Rust.

