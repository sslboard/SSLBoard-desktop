## Context

The current secret storage uses the `keyring` crate for cross-platform OS credential storage. On macOS, this provides basic Keychain integration but doesn't leverage Apple's biometric authentication features (Touch ID/Face ID). Users want the smooth biometric experience when accessing sensitive certificate secrets.

This change assumes backend-driven vault unlocking (see `update-vault-unlock-workflow`). With that workflow, biometric prompts appear automatically when operations require secrets, rather than requiring explicit unlock actions. This provides a natural macOS-like experience where authentication happens on-demand.

## Goals / Non-Goals

- Goals:
  - Enable Touch ID/Face ID authentication for accessing secrets on macOS
  - Maintain cross-platform compatibility (Windows/Linux unchanged)
  - Keep the same API surface for secret storage/retrieval
  - Preserve existing secrets and migration path

- Non-Goals:
  - Change the secret storage API or reference system
  - Implement biometric auth on non-macOS platforms
  - Require biometric hardware (graceful fallback if unavailable)
  - Change encryption or key management approach

## Decisions

- Decision: Use `security_framework` crate for direct macOS Keychain access with biometric protection
  - Why: `keyring` crate doesn't expose biometric access control APIs
  - Alternative considered: Extend `keyring` crate (not feasible, it's a cross-platform abstraction)

- Decision: Create platform-specific secret store implementations
  - Why: macOS gets biometric Keychain, others keep existing `keyring` implementation
  - Alternative considered: Always use `security_framework` (breaks cross-platform compatibility)

- Decision: Automatic biometric enrollment for new secrets on macOS
  - Why: Provides consistent security without user configuration burden
  - Alternative considered: Optional biometric setting (adds UX complexity)

- Decision: Biometric prompts appear on secret access, not on explicit unlock
  - Why: Aligns with backend-driven unlock workflow where vault unlocks automatically when secrets are needed
  - Why: Provides natural macOS-like experience where authentication happens on-demand
  - Alternative considered: Prompt on explicit unlock button (creates double-prompt UX issue)

## Risks / Trade-offs

- Risk: `security_framework` crate may have different API stability than `keyring`
  - Mitigation: Add comprehensive tests and monitor for breaking changes

- Risk: Biometric prompts may be disruptive if too frequent
  - Mitigation: Only protect the most sensitive secret types (ACME account keys, private keys)

- Risk: Users without biometric hardware get different experience
  - Mitigation: Graceful fallback to standard Keychain authentication

## Migration Plan

1. Existing secrets continue to work unchanged
2. New secrets on macOS automatically get biometric protection
3. No data migration required - existing keyring entries remain accessible
4. Users can upgrade to biometric protection by recreating sensitive secrets

## Implementation Approach

- Add conditional `security_framework` dependency (macOS only)
- Create `BiometricKeyringStore` alongside existing `KeyringSecretStore`
- Use runtime platform detection to select appropriate store
- Maintain same `SecretStore` trait interface for compatibility
