## 1. Dependencies & Setup

- [ ] 1.1 Add `security_framework` crate to Cargo.toml (macOS-only feature flag)
- [ ] 1.2 Update build configuration for conditional compilation
- [ ] 1.3 Verify macOS build compatibility

## 2. Core Implementation

- [ ] 2.1 Create `BiometricKeyringStore` in `src-tauri/src/secrets/biometric_store.rs`
- [ ] 2.2 Implement biometric access control using `security_framework::item::*`
- [ ] 2.3 Add biometric ItemAddOptions with Touch ID/Face ID protection
- [ ] 2.4 Implement store/retrieve/delete operations with biometric prompts

## 3. Platform Detection & Selection

- [ ] 3.1 Add platform detection logic in `secrets/mod.rs`
- [ ] 3.2 Create platform-specific store factory function
- [ ] 3.3 Update `SecretManager` to use platform-appropriate store
- [ ] 3.4 Maintain backward compatibility with existing `KeyringSecretStore`

## 4. Security & Error Handling

- [ ] 4.1 Add graceful fallback when biometric hardware unavailable
- [ ] 4.2 Handle biometric authentication failures appropriately
- [ ] 4.3 Ensure biometric protection only for sensitive secret types
- [ ] 4.4 Add security audit logging for biometric operations

## 5. Testing & Validation

- [ ] 5.1 Add unit tests for biometric store operations
- [ ] 5.2 Test biometric prompt behavior and user experience
- [ ] 5.3 Verify cross-platform compatibility (Windows/Linux unchanged)
- [ ] 5.4 Test migration path for existing secrets
- [ ] 5.5 Integration tests with certificate issuance flow

## 6. Documentation & UX

- [ ] 6.1 Update user-facing documentation about biometric protection
- [ ] 6.2 Add UI indicators for biometric-protected secrets
- [ ] 6.3 Document macOS-specific security enhancements
