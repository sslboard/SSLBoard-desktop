## MODIFIED Requirements

### Requirement: OS-backed secret storage
The system SHALL use the operating system's secure credential storage to protect secrets. On macOS, the system SHALL use biometric authentication (Touch ID/Face ID) for the master key where supported. On other platforms (Windows, Linux), the system SHALL use the standard OS credential storage (Credential Manager, Secret Service).

#### Scenario: macOS biometric protection for master key
- **WHEN** the master key is stored on macOS with biometric hardware available
- **THEN** the system SHALL create a Keychain item with access control requiring Touch ID, Face ID, or device passcode authentication
- **AND** the system SHALL use `AccessibleWhenPasscodeSetThisDeviceOnly` protection mode

#### Scenario: macOS biometric prompt on access
- **WHEN** an operation requires the master key on macOS with a biometric-protected keychain item
- **THEN** macOS SHALL automatically display the Touch ID/Face ID prompt
- **AND** successful authentication SHALL unlock the vault and allow the operation to proceed

#### Scenario: macOS passcode fallback
- **WHEN** biometric authentication fails or is cancelled on macOS
- **THEN** the system SHALL allow passcode authentication as fallback (handled by macOS)

#### Scenario: Graceful fallback when biometric hardware unavailable
- **WHEN** biometric hardware is not available on macOS (e.g., older Mac, Touch ID disabled)
- **THEN** the system SHALL fall back to standard Keychain authentication without biometric protection
- **AND** the master key SHALL still be stored securely in the Keychain

#### Scenario: Cross-platform compatibility maintained
- **WHEN** running on Windows or Linux
- **THEN** the system SHALL use the existing OS credential storage without biometric features
- **AND** the API surface SHALL remain unchanged

#### Scenario: Secret is durable across restarts
- **WHEN** a secret is stored and the application is restarted
- **THEN** the secret SHALL remain retrievable by reference id after unlocking the vault

## ADDED Requirements

### Requirement: Platform-specific secret store adapters
The system SHALL provide platform-specific secret store implementations that optimize security and user experience for each operating system while maintaining the same storage API.

#### Scenario: macOS uses biometric Keychain adapter
- **WHEN** running on macOS with biometric hardware available
- **THEN** the system SHALL use the `BiometricKeyringStore` adapter for the master key
- **AND** the adapter SHALL use `security_framework` crate for direct Keychain access

#### Scenario: macOS falls back to standard adapter
- **WHEN** running on macOS without biometric hardware
- **THEN** the system SHALL use the standard `MasterKeyStore` adapter
- **AND** secrets SHALL still be protected by the OS Keychain

#### Scenario: Windows/Linux use standard OS storage
- **WHEN** running on Windows or Linux
- **THEN** the system SHALL use the standard OS credential storage (Credential Manager, Secret Service)
- **AND** the `keyring` crate SHALL be used for cross-platform compatibility

### Requirement: Biometric access control configuration
The system SHALL configure biometric access control using Apple Security framework APIs with appropriate flags for secure, user-friendly authentication.

#### Scenario: Access control flags for biometric items
- **WHEN** creating a biometric-protected Keychain item
- **THEN** the system SHALL use `kSecAccessControlBiometryAny | kSecAccessControlOr | kSecAccessControlDevicePasscode` flags
- **AND** this SHALL allow any enrolled biometric OR device passcode for authentication

#### Scenario: Protection mode for sensitive items
- **WHEN** creating a biometric-protected Keychain item
- **THEN** the system SHALL use `AccessibleWhenPasscodeSetThisDeviceOnly` protection mode
- **AND** the item SHALL only be accessible when the device is unlocked
- **AND** the item SHALL NOT be migratable to other devices

#### Scenario: Biometric prompts appear on-demand
- **WHEN** a user performs an operation requiring secrets (e.g., issue certificate) and the vault is locked
- **THEN** the system SHALL automatically trigger vault unlock by accessing the master key from Keychain
- **AND** macOS SHALL display the biometric prompt automatically (no explicit unlock button needed)
- **AND** the operation SHALL proceed after successful biometric authentication
- **AND** the system SHALL fail gracefully if authentication is denied or unavailable
