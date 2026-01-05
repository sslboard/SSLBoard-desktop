# Change: Add macOS Biometric Keychain Storage

## Why

Users want the smooth Touch ID/Face ID experience when accessing sensitive certificate secrets, but the current cross-platform keyring approach doesn't provide biometric protection on macOS. Adding biometric access control to macOS Keychain items will enhance security while maintaining the seamless user experience that macOS users expect. This change works in conjunction with backend-driven vault unlocking, where biometric prompts appear automatically when secrets are accessed, providing a natural macOS-like authentication flow.

## What Changes

- Add `security_framework` v3.5 crate for macOS-specific Keychain operations (with `OSX_10_13` feature)
- Create `BiometricKeyringStore` implementing biometric access control for the master key
- Use `kSecAccessControlBiometryAny | kSecAccessControlOr | kSecAccessControlDevicePasscode` flags to allow Touch ID/Face ID or passcode
- Use `AccessibleWhenPasscodeSetThisDeviceOnly` protection mode for maximum security
- Create platform-specific store factory that selects the appropriate store at runtime
- Graceful fallback to standard `MasterKeyStore` when biometrics unavailable

## API Research Summary

The `security_framework` crate v3.5.1 provides:

- `SecAccessControl::create_with_protection(protection, flags)` - Creates access control with biometric flags
- `ProtectionMode::AccessibleWhenPasscodeSetThisDeviceOnly` - Best protection mode for biometric items
- `kSecAccessControlBiometryAny` (OSX_10_13+) - Allows any enrolled biometric
- `kSecAccessControlDevicePasscode` - Passcode fallback
- `kSecAccessControlOr` - Combine flags with OR logic

When a keychain item has biometric access control, macOS automatically displays the Touch ID/Face ID prompt when the item is accessed. No additional prompt logic is needed in our code.

See `design.md` for detailed API documentation and code examples.

## Impact

- Affected specs: `secret-store` (modify to add biometric protection)
- Affected code:
  - `src-tauri/Cargo.toml` - Add `security_framework` v3.5, `security_framework_sys` v2.15 (macOS only, OSX_10_13 feature)
  - `src-tauri/src/secrets/biometric_store.rs` - NEW: Biometric Keychain adapter
  - `src-tauri/src/secrets/mod.rs` - Platform detection and store factory
  - `src-tauri/src/secrets/manager.rs` - Use factory for store selection

This change is macOS-specific and maintains backward compatibility - existing secrets continue to work, new master keys on macOS with biometric hardware get biometric protection automatically.

**Dependencies**: This change works best with `update-vault-unlock-workflow` which implements backend-driven unlocking. With that workflow, biometric prompts appear automatically when secrets are accessed, providing a natural macOS-like experience.
