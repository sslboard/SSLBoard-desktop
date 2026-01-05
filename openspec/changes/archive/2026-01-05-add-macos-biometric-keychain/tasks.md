## 1. Dependencies & Setup

- [x] 1.1 Add `security-framework` v3.5 to Cargo.toml with `OSX_10_13` feature (macOS-only)
- [x] 1.2 Add `security-framework-sys` v2.15 with `OSX_10_13` feature (macOS-only)
- [x] 1.3 Add conditional compilation attributes for macOS-only code
- [x] 1.4 Verify macOS build compatibility with `cargo build --target x86_64-apple-darwin`
- [x] 1.5 Create `src-tauri/Entitlements.plist` (initially empty, for testing if needed)

## 2. Core Implementation

- [x] 2.1 Create `BiometricKeyringStore` struct in `src-tauri/src/secrets/biometric_store.rs`
- [x] 2.2 Implement `SecAccessControl` creation with biometric flags:
  - Use `kSecAccessControlBiometryAny | kSecAccessControlOr | kSecAccessControlDevicePasscode`
  - Use `ProtectionMode::AccessibleWhenPasscodeSetThisDeviceOnly`
- [x] 2.3 Implement `store_secret()` using `CFMutableDictionary` with `kSecAttrAccessControl`
- [x] 2.4 Implement `get_secret()` using `ItemSearchOptions` (biometric prompt handled by macOS)
- [x] 2.5 Implement `delete_secret()` for cleanup
- [x] 2.6 Add `check_biometric_available()` function (try-create `SecAccessControl`)

## 3. Platform Detection & Selection

- [x] 3.1 Add platform detection logic in `src-tauri/src/secrets/mod.rs`
- [x] 3.2 Create `create_master_key_store()` factory function
- [x] 3.3 On macOS: use `BiometricKeyringStore` if biometrics available, else `MasterKeyStore`
- [x] 3.4 On Windows/Linux: use existing `MasterKeyStore` unchanged
- [x] 3.5 Update `SecretManager` initialization to use factory function

## 4. Error Handling & Fallback

- [x] 4.1 Map `security_framework::base::Error` to `SecretStoreError`
- [x] 4.2 Add graceful fallback when `SecAccessControl::create_with_protection()` fails
- [x] 4.3 Handle user cancellation of biometric prompt (errSecUserCanceled)
- [x] 4.4 Handle authentication failure (errSecAuthFailed)
- [x] 4.5 Log biometric operations for debugging (use existing log macros)

## 5. Testing & Validation

- [x] 5.1 Add unit tests for `SecAccessControl` flag combinations
- [x] 5.2 Add macOS integration test: create biometric-protected keychain item
- [x] 5.3 Add macOS integration test: verify fallback when biometrics unavailable
- [x] 5.4 Add macOS integration test: verify item deletion cleans up properly
- [x] 5.5 Manual test: verify Touch ID prompt appears on secret access
- [x] 5.6 Manual test: verify passcode fallback works
- [x] 5.7 Manual test: verify works in Tauri production build (signed)
- [x] 5.8 Test on device without Touch ID (e.g., older Mac or VM)
- [x] 5.9 Test production build WITHOUT entitlements file first
- [x] 5.10 If `-34018` errors occur, add `Entitlements.plist` and reference in `tauri.conf.json`
- [x] 5.11 Document which entitlements are needed (if any) in final implementation

## 6. Documentation

- [x] 6.1 Add inline documentation to `BiometricKeyringStore`
- [x] 6.2 Document the flag choices and protection mode rationale
- [x] 6.3 Update `src-tauri/AGENTS.md` with biometric store notes
