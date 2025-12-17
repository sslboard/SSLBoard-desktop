## ADDED Requirements

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
The system SHALL use the operating system’s secure credential storage (macOS Keychain, Windows Credential Manager, Linux Secret Service) as the default backing store for `SecretStore` where available.

#### Scenario: Secret is durable across restarts
- **WHEN** a secret is stored and the application is restarted
- **THEN** the secret SHALL remain retrievable by reference id

#### Scenario: OS store unavailable fails fast
- **WHEN** the OS keychain/credential store is unavailable on the current platform
- **THEN** the system SHALL fail fast and return an error without falling back to a local file store

### Requirement: Supported secret kinds for v0
The system SHALL support storing secrets for at least these kinds:
- ACME account key references
- Managed private key references
- DNS provider credential references

#### Scenario: DNS credential ref is usable by the DNS adapter layer
- **WHEN** the DNS adapter layer is provided a DNS credential reference id
- **THEN** it SHALL be able to resolve the credential inside Rust without involving the UI

### Requirement: Secret references are prefixed and stable
Secret reference identifiers MUST be non-sequential, prefixed (e.g., `sec_`), and stable for the lifetime of the secret entry.

#### Scenario: Reference id format is enforced
- **WHEN** a secret reference id is generated
- **THEN** it SHALL begin with the configured prefix and be suitable for long-term reuse across modules

### Requirement: Secret metadata is non-secret and stored in the DB
The system SHALL persist only non-secret metadata (e.g., ref id, kind, user-provided label, createdAt) in the local database so the UI can list and manage secrets without accessing secret bytes.

#### Scenario: UI lists secrets using metadata only
- **WHEN** the UI requests the “Secrets” list
- **THEN** it SHALL receive only the metadata from the database (including label for DNS credentials) and SHALL NOT receive secret values

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

