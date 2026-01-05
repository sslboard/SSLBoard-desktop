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

## API Research: security_framework v3.5

### Crate Overview

The `security_framework` crate (v3.5.1) provides Rust bindings to Apple's Security Framework. Key modules:

- `security_framework::access_control` - Access control settings for keychain items
- `security_framework::item` - Keychain item CRUD operations
- `security_framework_sys::access_control` - Low-level FFI bindings with biometric flags

### Key Types and APIs

#### SecAccessControl

```rust
use security_framework::access_control::{SecAccessControl, ProtectionMode};
use core_foundation_sys::base::CFOptionFlags;

// Create access control with protection mode and flags
let access_control = SecAccessControl::create_with_protection(
    Some(ProtectionMode::AccessibleWhenPasscodeSetThisDeviceOnly),
    flags,  // CFOptionFlags - see biometric flags below
)?;
```

**Methods:**

- `create_with_flags(flags: CFOptionFlags) -> Result<Self>` - Create from flags only
- `create_with_protection(protection: Option<ProtectionMode>, flags: CFOptionFlags) -> Result<Self>` - Create with protection mode and flags

#### ProtectionMode Enum

Specifies when an item is available:

| Variant | Description |
|---------|-------------|
| `AccessibleWhenPasscodeSetThisDeviceOnly` | Only when device is unlocked AND passcode is set (recommended for biometric) |
| `AccessibleWhenUnlockedThisDeviceOnly` | Only when device is unlocked, this device only |
| `AccessibleWhenUnlocked` | Only when device is unlocked (can sync) |
| `AccessibleAfterFirstUnlockThisDeviceOnly` | After first unlock, this device only |
| `AccessibleAfterFirstUnlock` | After first unlock (can sync) |

**Recommendation:** Use `AccessibleWhenPasscodeSetThisDeviceOnly` for biometric-protected items. This ensures:

1. Device must have a passcode set (required for biometrics)
2. Item is only accessible when device is unlocked
3. Item cannot be migrated to other devices

### Biometric Access Control Flags

From `security_framework_sys::access_control`:

```rust
// Available in all versions
pub const kSecAccessControlUserPresence: CFOptionFlags = 1 << 0;

// Requires feature = "OSX_10_13" (macOS 10.13+)
pub const kSecAccessControlBiometryAny: CFOptionFlags = 1 << 1;
pub const kSecAccessControlBiometryCurrentSet: CFOptionFlags = 1 << 3;

// Other flags
pub const kSecAccessControlDevicePasscode: CFOptionFlags = 1 << 4;
pub const kSecAccessControlOr: CFOptionFlags = 1 << 14;
pub const kSecAccessControlAnd: CFOptionFlags = 1 << 15;
```

**Flag Descriptions:**

| Flag | Description | Use Case |
|------|-------------|----------|
| `kSecAccessControlUserPresence` | Requires user presence (biometric or passcode) | General user verification |
| `kSecAccessControlBiometryAny` | Any enrolled biometric (persists if biometrics change) | Less strict biometric |
| `kSecAccessControlBiometryCurrentSet` | Current biometric enrollment only (invalidated if biometrics change) | Most secure biometric |
| `kSecAccessControlDevicePasscode` | Passcode only, no biometric | Passcode fallback |
| `kSecAccessControlOr` | Combine with OR (any condition) | Flexible access |
| `kSecAccessControlAnd` | Combine with AND (all conditions) | Strict access |

**Recommended Flags for SSLBoard:**

```rust
// Primary: Biometric with passcode fallback
let flags = kSecAccessControlBiometryAny | kSecAccessControlOr | kSecAccessControlDevicePasscode;

// Alternative: Current biometric set only (more secure, but invalidated if fingerprints change)
let flags = kSecAccessControlBiometryCurrentSet;
```

### Working with Keychain Items

#### Creating an Access-Controlled Keychain Item

The `security_framework` crate's `ItemAddOptions` doesn't directly support `kSecAttrAccessControl`. To set biometric access control, you need to use the lower-level dictionary API:

```rust
use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFMutableDictionary;
use core_foundation::string::CFString;
use core_foundation::data::CFData;
use security_framework::access_control::{SecAccessControl, ProtectionMode};
use security_framework_sys::access_control::{kSecAccessControlBiometryAny, kSecAccessControlOr, kSecAccessControlDevicePasscode};
use security_framework_sys::item::{kSecClass, kSecClassGenericPassword, kSecAttrService, kSecAttrAccount, kSecValueData, kSecAttrAccessControl};

fn add_biometric_protected_item(
    service: &str,
    account: &str,
    data: &[u8],
) -> Result<(), Error> {
    // Create access control with biometric protection
    let flags = kSecAccessControlBiometryAny | kSecAccessControlOr | kSecAccessControlDevicePasscode;
    let access_control = SecAccessControl::create_with_protection(
        Some(ProtectionMode::AccessibleWhenPasscodeSetThisDeviceOnly),
        flags,
    )?;

    // Build the item dictionary
    let mut dict = CFMutableDictionary::new();
    
    unsafe {
        dict.set(
            CFType::wrap_under_get_rule(kSecClass as *const _),
            CFType::wrap_under_get_rule(kSecClassGenericPassword as *const _),
        );
        dict.set(
            CFString::new("kSecAttrService").as_CFType(),
            CFString::new(service).as_CFType(),
        );
        dict.set(
            CFString::new("kSecAttrAccount").as_CFType(),
            CFString::new(account).as_CFType(),
        );
        dict.set(
            CFString::new("kSecValueData").as_CFType(),
            CFData::from_buffer(data).as_CFType(),
        );
        dict.set(
            CFString::new("kSecAttrAccessControl").as_CFType(),
            access_control.as_CFType(),
        );
    }

    // Add to keychain
    let status = unsafe { SecItemAdd(dict.as_concrete_TypeRef(), std::ptr::null_mut()) };
    
    if status == errSecSuccess {
        Ok(())
    } else {
        Err(Error::from_code(status))
    }
}
```

#### Retrieving a Biometric-Protected Item

When retrieving a biometric-protected item, macOS automatically prompts for Touch ID/Face ID:

```rust
use security_framework::item::{ItemSearchOptions, ItemClass, SearchResult};

fn get_biometric_protected_item(service: &str, account: &str) -> Result<Vec<u8>, Error> {
    // Standard keychain search - macOS handles biometric prompt automatically
    let results = ItemSearchOptions::new()
        .class(ItemClass::generic_password())
        .service(service)
        .account(account)
        .load_data(true)
        .search()?;

    match results.first() {
        Some(SearchResult::Data(data)) => Ok(data.to_vec()),
        _ => Err(Error::NotFound),
    }
}
```

### Checking Biometric Availability

To check if biometrics are available before attempting to use them:

```rust
// Option 1: Try-and-fallback approach (simpler)
// Just attempt to create biometric access control and handle errors

// Option 2: Use LocalAuthentication framework (more complex)
// This requires binding to LAContext which isn't in security_framework
// Would need objc crate or custom FFI:

#[cfg(target_os = "macos")]
fn is_biometric_available() -> bool {
    use objc::{class, msg_send, sel, sel_impl};
    use objc::runtime::Object;
    
    unsafe {
        let la_context: *mut Object = msg_send![class!(LAContext), new];
        let mut error: *mut Object = std::ptr::null_mut();
        let result: bool = msg_send![
            la_context, 
            canEvaluatePolicy: 1 // LAPolicyDeviceOwnerAuthenticationWithBiometrics
            error: &mut error
        ];
        let _: () = msg_send![la_context, release];
        result
    }
}
```

**Recommendation:** Use the try-and-fallback approach for simplicity. Create access control with biometric flags, and if it fails or the item can't be accessed, fall back to standard keychain access.

### Feature Flags Required

In `Cargo.toml`:

```toml
[target.'cfg(target_os = "macos")'.dependencies]
security-framework = { version = "3.5", features = ["OSX_10_13"] }
security-framework-sys = { version = "2.15", features = ["OSX_10_13"] }
```

The `OSX_10_13` feature enables:

- `kSecAccessControlBiometryAny`
- `kSecAccessControlBiometryCurrentSet`

Without this feature, only `kSecAccessControlUserPresence` is available, which still works for biometric authentication but is less specific.

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

- Decision: Use `kSecAccessControlBiometryAny | kSecAccessControlOr | kSecAccessControlDevicePasscode` as default flags
  - Why: Allows biometric or passcode, persists across biometric changes
  - Alternative considered: `kSecAccessControlBiometryCurrentSet` (more secure but invalidated if user adds/removes fingerprints)

- Decision: Use `AccessibleWhenPasscodeSetThisDeviceOnly` protection mode
  - Why: Requires passcode (prerequisite for biometrics), device-only, maximum security
  - Alternative considered: `AccessibleWhenUnlocked` (less secure, can migrate)

## Risks / Trade-offs

- Risk: `security_framework` crate may have different API stability than `keyring`
  - Mitigation: Add comprehensive tests and monitor for breaking changes

- Risk: Biometric prompts may be disruptive if too frequent
  - Mitigation: Only protect the most sensitive secret types (ACME account keys, private keys)

- Risk: Users without biometric hardware get different experience
  - Mitigation: Graceful fallback to standard Keychain authentication (passcode)

- Risk: Prior implementation attempts did not trigger biometric prompts or report availability
  - Mitigation: Add macOS integration tests to validate availability checks and prompt behavior

- Risk: Tauri sandbox may restrict keychain access
  - Mitigation: Test in sandboxed Tauri environment; may need entitlements

## Implementation Approach

### File Structure

```
src-tauri/src/secrets/
├── mod.rs                    # Platform detection and store factory
├── store.rs                  # SecretStore trait (existing)
├── keyring_store.rs          # Existing cross-platform store
├── biometric_store.rs        # NEW: macOS biometric keychain store
└── manager.rs                # SecretManager (uses store factory)
```

### BiometricKeyringStore Implementation Sketch

```rust
// src-tauri/src/secrets/biometric_store.rs
#![cfg(target_os = "macos")]

use security_framework::access_control::{SecAccessControl, ProtectionMode};
use security_framework_sys::access_control::*;
use super::store::{SecretStore, SecretStoreError};

pub struct BiometricKeyringStore {
    service: String,
}

impl BiometricKeyringStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self { service: service.into() }
    }
    
    fn create_biometric_access_control() -> Result<SecAccessControl, SecretStoreError> {
        let flags = kSecAccessControlBiometryAny 
            | kSecAccessControlOr 
            | kSecAccessControlDevicePasscode;
        
        SecAccessControl::create_with_protection(
            Some(ProtectionMode::AccessibleWhenPasscodeSetThisDeviceOnly),
            flags,
        ).map_err(|e| SecretStoreError::Store(format!("Failed to create access control: {}", e)))
    }
    
    // ... implement SecretStore trait
}
```

### Platform Detection

```rust
// src-tauri/src/secrets/mod.rs

#[cfg(target_os = "macos")]
mod biometric_store;

pub fn create_secret_store(service: &str) -> Box<dyn SecretStore> {
    #[cfg(target_os = "macos")]
    {
        // Try biometric store first, fall back to keyring
        if is_biometric_supported() {
            return Box::new(biometric_store::BiometricKeyringStore::new(service));
        }
    }
    
    Box::new(keyring_store::KeyringSecretStore::new(service))
}

#[cfg(target_os = "macos")]
fn is_biometric_supported() -> bool {
    // Simple heuristic: try to create biometric access control
    // If it succeeds, biometrics are likely available
    biometric_store::BiometricKeyringStore::check_biometric_available()
}
```

## Migration Plan

1. Existing secrets continue to work unchanged (stored via `keyring`)
2. New secrets on macOS automatically get biometric protection
3. No data migration required - existing keyring entries remain accessible
4. Users can upgrade to biometric protection by recreating sensitive secrets

## Testing Strategy

### Unit Tests

- Test `SecAccessControl` creation with various flag combinations
- Test error handling for unavailable biometrics
- Test protection mode selection

### Integration Tests (macOS only)

- Test adding a biometric-protected keychain item
- Test retrieving a biometric-protected item (requires manual Touch ID interaction)
- Test fallback behavior when biometrics unavailable
- Test behavior in Tauri sandboxed environment

### Manual Testing Checklist

- [ ] Touch ID prompt appears when accessing protected secrets
- [ ] Passcode fallback works if Touch ID fails
- [ ] Existing non-biometric secrets remain accessible
- [ ] Graceful degradation on devices without Touch ID
- [ ] Works correctly in Tauri production build

## macOS Entitlements

### Do We Need `keychain-access-groups`?

**Short answer: No, not for basic biometric keychain access.**

The `keychain-access-groups` entitlement is for **sharing keychain items between multiple apps** from the same developer (e.g., SSLBoard and SSLBoard Pro sharing credentials). Since SSLBoard only needs to access its own keychain items, the default access group (app's bundle identifier: `com.sslboard.desktop`) is sufficient.

### What Entitlements Might Be Needed?

For a signed, hardened runtime macOS app, the biometric keychain should work without special entitlements because:

1. The app is properly code-signed (configured in `tauri.conf.json`)
2. The app only accesses its own keychain items (default access group)
3. Biometric prompts are handled by macOS, not the app

However, if issues arise, create an entitlements file at `src-tauri/Entitlements.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Required for hardened runtime with keychain access -->
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <false/>
</dict>
</plist>
```

Then reference it in `tauri.conf.json`:

```json
"macOS": {
  "entitlements": "Entitlements.plist"
}
```

### Potential Error Codes

| Error Code | Meaning | Solution |
|------------|---------|----------|
| `-34018` | Missing entitlements | Add entitlements file |
| `-25293` | Keychain locked | User needs to unlock keychain |
| `-25291` | User cancelled | Handle gracefully in code |
| `-25300` | Item not found | Expected for new installations |

### Testing Without Entitlements First

The recommended approach:

1. **Development builds** (`npm run tauri dev`) - Should work without entitlements
2. **Production builds** (`npm run tauri build`) - Test with and without entitlements
3. **If `-34018` errors occur** - Add the entitlements file

## Open Questions (Resolved)

- ✅ Do `security_framework` access-control settings reliably trigger Touch ID/Face ID prompts when reading Keychain items?
  - **Answer:** Yes, when `kSecAttrAccessControl` is set with biometric flags, macOS automatically prompts for biometrics on item access.

- ✅ What is the most reliable availability signal for biometrics on macOS in this app context?
  - **Answer:** Use try-and-fallback approach. Attempt to create `SecAccessControl` with biometric flags; if it succeeds, biometrics are likely available. For precise detection, use `LAContext.canEvaluatePolicy()` via objc bindings.

- ✅ Does the Tauri runtime or sandboxing affect biometric prompt availability?
  - **Answer:** Likely no special entitlements needed. The `keychain-access-groups` entitlement is only for sharing items between apps. SSLBoard uses its own keychain items with the default access group (bundle ID). If issues occur, add a minimal entitlements file. See "macOS Entitlements" section above.
