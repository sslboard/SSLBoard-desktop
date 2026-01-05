## MODIFIED Requirements

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

