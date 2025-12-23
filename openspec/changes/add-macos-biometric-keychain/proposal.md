# Change: Add macOS Biometric Keychain Storage

## Why

Users want the smooth Touch ID/Face ID experience when accessing sensitive certificate secrets, but the current cross-platform keyring approach doesn't provide biometric protection on macOS. Adding biometric access control to macOS Keychain items will enhance security while maintaining the seamless user experience that macOS users expect.

## What Changes

- Add `security_framework` crate for macOS-specific Keychain operations
- Implement biometric access control (Touch ID/Face ID) for sensitive secrets on macOS
- Create platform-specific secret store adapters (macOS with biometrics, others unchanged)
- Modify secret store manager to use platform-specific adapters
- Keep cross-platform compatibility for Windows/Linux while enhancing macOS UX

## Impact

- Affected specs: `secret-store` (modify to add biometric protection)
- Affected code:
  - `src-tauri/Cargo.toml` - Add `security_framework` dependency (macOS only)
  - `src-tauri/src/secrets/keyring_store.rs` - Add biometric Keychain adapter
  - `src-tauri/src/secrets/manager.rs` - Platform-specific adapter selection
  - `src-tauri/src/secrets/mod.rs` - Export new biometric store

This change is macOS-specific and maintains backward compatibility - existing secrets continue to work, new secrets on macOS get biometric protection automatically.
