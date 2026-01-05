# Secret Store Specification

## Purpose

The Secret Store provides a secure, local-only abstraction for managing sensitive credentials (DNS API tokens, ACME account keys, private keys) without exposing them to the untrusted UI. It leverages OS-level secure storage (Keychain, Credential Manager) and local encryption to ensure secrets remain protected at rest and in memory.

## Requirements

### Requirement: Secrets remain in the trusted Rust core

The system MUST ensure that raw secret material (DNS API tokens, ACME account private keys, managed private keys) never crosses the IPC boundary to the UI and is only accepted by Rust during create/update flows.

#### Scenario: UI can manage secrets without seeing secret bytes

- **WHEN** the user creates or updates a DNS credential via the UI
- **THEN** the UI SHALL send the secret value to Rust once and SHALL receive only a secret reference id and non-sensitive metadata in response

### Requirement: SecretStore abstraction

The system SHALL provide a `secrets::store::SecretStore` abstraction that supports storing and retrieving secrets by reference id within the Rust core.

#### Scenario: Secret retrieved inside Rust by reference id

- **WHEN** a Rust module requests a secret by a valid reference id
- **THEN** the secret store SHALL return the secret value to Rust

#### Scenario: Secret reference id not found

- **WHEN** a Rust module requests a secret by an unknown reference id
- **THEN** the secret store SHALL return a not-found error

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

### Requirement: Supported secret kinds for v0

The system SHALL support storing secrets for at least these kinds:

- ACME account key references
- Managed private key references
- DNS provider API token references (owned by DnsProvider entities, not standalone)

#### Scenario: DNS provider token stored via provider creation flow

- **WHEN** the user creates a DNS provider with an API token
- **THEN** the system SHALL store the token as a secret with kind "dns_provider_token" and link it to the provider via `secret_ref`

#### Scenario: DNS provider token deleted with provider

- **WHEN** the user deletes a DNS provider
- **THEN** the system SHALL also delete the associated secret from SecretStore

### Requirement: Secret references are prefixed and stable

Secret reference identifiers MUST be non-sequential, prefixed (e.g., `sec_`), and stable for the lifetime of the secret entry.

#### Scenario: Reference id format is enforced

- **WHEN** a secret reference id is generated
- **THEN** it SHALL begin with the configured prefix and be suitable for long-term reuse across modules

### Requirement: Secret metadata is non-secret and stored in the DB

The system SHALL persist non-secret metadata (ref id, kind, user-provided label, createdAt) along with the encrypted secret ciphertext in the local database. The UI can list and manage secrets using metadata without accessing decrypted secret bytes.

#### Scenario: UI lists secrets using metadata only

- **WHEN** the UI requests the "Secrets" list
- **THEN** it SHALL receive only the metadata from the database (including label for DNS credentials) and SHALL NOT receive decrypted secret values

#### Scenario: Ciphertext stored alongside metadata

- **WHEN** a secret is created or updated
- **THEN** the encrypted ciphertext SHALL be stored in the `secret_metadata` table alongside the metadata fields

### Requirement: Secret value replacement keeps reference id

The system SHALL allow replacing a secret’s value while preserving its reference id so dependent records remain valid.

#### Scenario: Secret is updated without changing reference

- **WHEN** the user updates a secret value via the UI
- **THEN** Rust SHALL overwrite the stored secret while keeping the same reference id and returning only metadata to the UI

### Requirement: Secret reference management UI

The UI SHALL provide a “Settings → Secrets” view that lists secret references and allows adding and removing them without displaying secret values.

#### Scenario: Removing a secret reference

- **WHEN** the user removes a secret reference id
- **THEN** the secret SHALL no longer be retrievable within Rust by that reference id

### Requirement: Master key generation and storage

The system SHALL generate a 256-bit (32-byte) cryptographically random master key on first use and store it in the OS keyring under a well-known service/user identifier.

#### Scenario: First-time master key creation

- **WHEN** the application starts and no master key exists in the keyring
- **THEN** the system SHALL generate a new 256-bit random key and store it in the OS keyring

#### Scenario: Master key is retrieved on unlock

- **WHEN** the vault is unlocked
- **THEN** the master key SHALL be retrieved from the OS keyring and cached in memory

### Requirement: Vault lock and unlock API

The system SHALL provide explicit `lock()` function to control master key availability in memory, enabling user-initiated locking. The system SHALL automatically unlock the vault on-demand when operations require secrets, triggering appropriate authentication (biometric, keyring password, etc.) as needed.

#### Scenario: Lock wipes master key from memory

- **WHEN** the `lock()` function is called
- **THEN** the cached master key SHALL be securely zeroized from memory
- **AND** subsequent secret operations SHALL automatically trigger unlock with authentication

#### Scenario: Automatic unlock on secret access

- **WHEN** an operation requires secrets and the vault is locked
- **THEN** the system SHALL automatically unlock the vault by retrieving the master key from the OS keyring
- **AND** the system SHALL trigger appropriate authentication (biometric prompt, keyring password, etc.) as required by the OS
- **AND** the vault SHALL be unlocked and the operation SHALL proceed after successful authentication

#### Scenario: Vault state is queryable

- **WHEN** a module needs to check vault status
- **THEN** the system SHALL provide an `is_unlocked()` function returning the current lock state

#### Scenario: User can explicitly lock vault

- **WHEN** the user initiates a lock action via the UI
- **THEN** the system SHALL lock the vault and zeroize the master key from memory
- **AND** the UI SHALL display the locked state

#### Scenario: Locked state displayed without requiring unlock

- **WHEN** the vault is locked
- **THEN** the UI SHALL display the locked state
- **AND** the UI SHALL NOT require the user to explicitly unlock before performing operations
- **AND** unlock SHALL happen automatically when operations require secrets

### Requirement: Master key secure memory handling

The system SHALL use secure memory handling for the master key, ensuring the key is zeroized when no longer needed to minimize exposure in memory.

#### Scenario: Master key zeroized on lock

- **WHEN** the vault is locked
- **THEN** the master key bytes SHALL be overwritten with zeros before being deallocated

#### Scenario: Master key zeroized on application exit

- **WHEN** the application terminates
- **THEN** the master key SHALL be zeroized as part of cleanup

### Requirement: Secrets database strict file permissions

The system SHALL create and maintain the `secrets.sqlite` database file with strict file permissions (mode 0600 on Unix systems) to prevent unauthorized access.

#### Scenario: Database created with restricted permissions

- **WHEN** the secrets database file is created for the first time
- **THEN** it SHALL have file permissions set to owner read/write only (0600)

#### Scenario: Permissions corrected on startup

- **WHEN** the application starts and the secrets database has overly permissive permissions
- **THEN** the system SHALL restrict the permissions to 0600

### Requirement: Migration from per-secret keyring storage

The system SHALL migrate existing secrets from per-secret keyring entries to the new master-key-encrypted SQLite storage on upgrade.

#### Scenario: Existing secrets are migrated

- **WHEN** the application detects secrets in the old per-secret keyring format
- **THEN** it SHALL encrypt each secret with the master key and store it in SQLite
- **AND** delete the old keyring entry after successful migration

#### Scenario: Orphaned secret metadata is cleaned up

- **WHEN** a secret metadata row exists but the corresponding keyring entry is missing
- **THEN** the system SHALL silently delete the orphaned metadata row

#### Scenario: Migration is idempotent

- **WHEN** migration has already been completed
- **THEN** subsequent application starts SHALL NOT re-run migration logic

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

