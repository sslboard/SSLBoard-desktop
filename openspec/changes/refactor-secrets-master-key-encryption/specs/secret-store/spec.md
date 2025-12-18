## MODIFIED Requirements

### Requirement: OS-backed secret storage
The system SHALL use the operating system's secure credential storage (macOS Keychain, Windows Credential Manager, Linux Secret Service) to store a single master encryption key. Individual secrets SHALL be encrypted with this master key using AES-256-GCM and stored in the local SQLite database.

#### Scenario: Secret is durable across restarts
- **WHEN** a secret is stored and the application is restarted
- **THEN** the secret SHALL remain retrievable by reference id after unlocking the vault

#### Scenario: Master key unavailable fails gracefully
- **WHEN** the OS keyring is unavailable or the master key entry is missing
- **THEN** the system SHALL return an appropriate error indicating the vault cannot be unlocked

#### Scenario: Secrets are encrypted at rest
- **WHEN** a secret is stored
- **THEN** the secret value SHALL be encrypted with AES-256-GCM using the master key before being written to SQLite

## ADDED Requirements

### Requirement: Master key generation and storage
The system SHALL generate a 256-bit (32-byte) cryptographically random master key on first use and store it in the OS keyring under a well-known service/user identifier.

#### Scenario: First-time master key creation
- **WHEN** the application starts and no master key exists in the keyring
- **THEN** the system SHALL generate a new 256-bit random key and store it in the OS keyring

#### Scenario: Master key is retrieved on unlock
- **WHEN** the vault is unlocked
- **THEN** the master key SHALL be retrieved from the OS keyring and cached in memory

### Requirement: Vault lock and unlock API
The system SHALL provide explicit `lock()` and `unlock()` functions to control master key availability in memory, enabling a user-facing lock/unlock feature.

#### Scenario: Lock wipes master key from memory
- **WHEN** the `lock()` function is called
- **THEN** the cached master key SHALL be securely zeroized from memory
- **AND** subsequent secret operations SHALL fail until `unlock()` is called

#### Scenario: Unlock retrieves and caches master key
- **WHEN** the `unlock()` function is called
- **THEN** the master key SHALL be retrieved from the OS keyring and cached for secret operations

#### Scenario: Vault state is queryable
- **WHEN** a module needs to check vault status
- **THEN** the system SHALL provide an `is_unlocked()` function returning the current lock state

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

## MODIFIED Requirements

### Requirement: Secret metadata is non-secret and stored in the DB
The system SHALL persist non-secret metadata (ref id, kind, user-provided label, createdAt) along with the encrypted secret ciphertext in the local database. The UI can list and manage secrets using metadata without accessing decrypted secret bytes.

#### Scenario: UI lists secrets using metadata only
- **WHEN** the UI requests the "Secrets" list
- **THEN** it SHALL receive only the metadata from the database (including label for DNS credentials) and SHALL NOT receive decrypted secret values

#### Scenario: Ciphertext stored alongside metadata
- **WHEN** a secret is created or updated
- **THEN** the encrypted ciphertext SHALL be stored in the `secret_metadata` table alongside the metadata fields

