## 1. Dependencies & Setup

- [ ] 1.1 Add `security-framework` v3.5 to Cargo.toml with `OSX_10_13` feature (macOS-only)
- [ ] 1.2 Add `security-framework-sys` v2.15 with `OSX_10_13` feature (macOS-only)
- [ ] 1.3 Add conditional compilation attributes for macOS-only code
- [ ] 1.4 Verify macOS build compatibility with `cargo build --target x86_64-apple-darwin`
- [ ] 1.5 Create `src-tauri/Entitlements.plist` (initially empty, for testing if needed)

## 2. Core Implementation

- [ ] 2.1 Create `BiometricKeyringStore` struct in `src-tauri/src/secrets/biometric_store.rs`
- [ ] 2.2 Implement `SecAccessControl` creation with biometric flags:
  - Use `kSecAccessControlBiometryAny | kSecAccessControlOr | kSecAccessControlDevicePasscode`
  - Use `ProtectionMode::AccessibleWhenPasscodeSetThisDeviceOnly`
- [ ] 2.3 Implement `store_secret()` using `CFMutableDictionary` with `kSecAttrAccessControl`
- [ ] 2.4 Implement `get_secret()` using `ItemSearchOptions` (biometric prompt handled by macOS)
- [ ] 2.5 Implement `delete_secret()` for cleanup
- [ ] 2.6 Add `check_biometric_available()` function (try-create `SecAccessControl`)

## 3. Platform Detection & Selection

- [ ] 3.1 Add platform detection logic in `src-tauri/src/secrets/mod.rs`
- [ ] 3.2 Create `create_master_key_store()` factory function
- [ ] 3.3 On macOS: use `BiometricKeyringStore` if biometrics available, else `MasterKeyStore`
- [ ] 3.4 On Windows/Linux: use existing `MasterKeyStore` unchanged
- [ ] 3.5 Update `SecretManager` initialization to use factory function

## 4. Error Handling & Fallback

- [ ] 4.1 Map `security_framework::base::Error` to `SecretStoreError`
- [ ] 4.2 Add graceful fallback when `SecAccessControl::create_with_protection()` fails
- [ ] 4.3 Handle user cancellation of biometric prompt (errSecUserCanceled)
- [ ] 4.4 Handle authentication failure (errSecAuthFailed)
- [ ] 4.5 Log biometric operations for debugging (use existing log macros)

## 5. Testing & Validation

- [ ] 5.1 Add unit tests for `SecAccessControl` flag combinations
- [ ] 5.2 Add macOS integration test: create biometric-protected keychain item
- [ ] 5.3 Add macOS integration test: verify fallback when biometrics unavailable
- [ ] 5.4 Add macOS integration test: verify item deletion cleans up properly
- [ ] 5.5 Manual test: verify Touch ID prompt appears on secret access
- [ ] 5.6 Manual test: verify passcode fallback works
- [ ] 5.7 Manual test: verify works in Tauri production build (signed)
- [ ] 5.8 Test on device without Touch ID (e.g., older Mac or VM)
- [ ] 5.9 Test production build WITHOUT entitlements file first
- [ ] 5.10 If `-34018` errors occur, add `Entitlements.plist` and reference in `tauri.conf.json`
- [ ] 5.11 Document which entitlements are needed (if any) in final implementation

## 6. Documentation

- [ ] 6.1 Add inline documentation to `BiometricKeyringStore`
- [ ] 6.2 Document the flag choices and protection mode rationale
- [ ] 6.3 Update `src-tauri/AGENTS.md` with biometric store notes
