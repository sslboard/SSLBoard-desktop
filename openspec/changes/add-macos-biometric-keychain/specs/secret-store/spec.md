## MODIFIED Requirements

### Requirement: OS-backed secret storage
The system SHALL use the operating system's secure credential storage to protect secrets. On macOS, the system SHALL use biometric authentication (Touch ID/Face ID) for sensitive secrets where supported. On other platforms (Windows, Linux), the system SHALL use the standard OS credential storage (Credential Manager, Secret Service).

#### Scenario: macOS biometric protection for sensitive secrets
- **WHEN** a sensitive secret (ACME account key, private key) is stored on macOS
- **THEN** the system SHALL create a Keychain item with biometric access control requiring Touch ID or Face ID authentication

#### Scenario: Graceful fallback when biometric hardware unavailable
- **WHEN** biometric hardware is not available on macOS
- **THEN** the system SHALL fall back to standard Keychain authentication without biometric protection

#### Scenario: Cross-platform compatibility maintained
- **WHEN** running on Windows or Linux
- **THEN** the system SHALL use the existing OS credential storage without biometric features

#### Scenario: Secret is durable across restarts
- **WHEN** a secret is stored and the application is restarted
- **THEN** the secret SHALL remain retrievable by reference id after unlocking the vault

## ADDED Requirements

### Requirement: Platform-specific secret store adapters
The system SHALL provide platform-specific secret store implementations that optimize security and user experience for each operating system while maintaining the same storage API.

#### Scenario: macOS uses biometric Keychain adapter
- **WHEN** running on macOS
- **THEN** the system SHALL use the biometric Keychain adapter for sensitive secret types
- **AND** standard secrets SHALL continue using the existing keyring adapter

#### Scenario: Windows/Linux use standard OS storage
- **WHEN** running on Windows or Linux
- **THEN** the system SHALL use the standard OS credential storage (Credential Manager, Secret Service)

### Requirement: Biometric access control configuration
The system SHALL automatically configure biometric access control for sensitive secret types on macOS, requiring user authentication before secret access.

#### Scenario: Automatic biometric enrollment for sensitive secrets
- **WHEN** storing an ACME account key or private key on macOS
- **THEN** the system SHALL automatically configure Touch ID/Face ID protection
- **AND** display appropriate user prompts when accessing the secret

#### Scenario: Biometric authentication required for secret access
- **WHEN** an operation requires a biometric-protected secret on macOS and the vault is locked
- **THEN** the system SHALL automatically unlock the vault by accessing the master key from Keychain
- **AND** macOS SHALL prompt for Touch ID or Face ID authentication when accessing the Keychain item
- **AND** after successful authentication, the vault SHALL be unlocked and the operation SHALL proceed
- **AND** the system SHALL fail gracefully if authentication is denied or unavailable

#### Scenario: Biometric prompt appears on-demand during operations
- **WHEN** a user performs an operation requiring secrets (e.g., issue certificate) and the vault is locked
- **THEN** the system SHALL automatically trigger vault unlock
- **AND** macOS SHALL display the biometric prompt automatically (no explicit unlock button needed)
- **AND** the operation SHALL proceed after successful biometric authentication
