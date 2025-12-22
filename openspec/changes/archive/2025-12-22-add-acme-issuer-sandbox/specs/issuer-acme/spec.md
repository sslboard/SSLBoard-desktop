## ADDED Requirements

### Requirement: Pluggable issuer interface
The system SHALL provide an issuer interface in the trusted Rust core that supports, at minimum: `ensure_account()`, `begin_order(domains)`, `get_challenges(order)`, `finalize(order, csr)`, and `download_certificate(order)`.

#### Scenario: Issuer can begin an order for a set of domains
- **WHEN** the issuance engine requests `begin_order(domains)` with one or more domain names
- **THEN** the issuer SHALL create an order and return an order identifier

### Requirement: ACME issuer defaults to Let’s Encrypt staging
The system SHALL provide an ACME issuer implementation configured to use Let’s Encrypt **staging** as the default endpoint.

#### Scenario: Sandbox is the default
- **WHEN** the user has not explicitly selected a production issuer endpoint
- **THEN** the system SHALL use the staging issuer configuration by default

### Requirement: ACME account key stored as a secret reference
The system MUST store the ACME account private key in `SecretStore` and MUST NOT expose the key material to the UI; the UI and metadata storage SHALL use only a secret reference id.

#### Scenario: Account creation persists key reference
- **WHEN** `ensure_account()` creates a new ACME account
- **THEN** the system SHALL persist a secret reference id for the account key for future use

#### Scenario: Account creation is idempotent
- **WHEN** `ensure_account()` is invoked and an account key reference already exists
- **THEN** the system SHALL reuse the existing account and SHALL NOT create a new key

### Requirement: Issuer configuration is persisted
The system SHALL persist the selected `issuer_id` and issuer configuration in the local non-secret metadata store.

#### Scenario: Issuer selection persists across restarts
- **WHEN** the user selects the staging issuer in settings
- **THEN** the selection SHALL persist after application restart

### Requirement: UI clearly indicates sandbox issuer
The UI SHALL make it visually obvious when the system is configured to use a sandbox/staging issuer endpoint.

#### Scenario: Sandbox banner is visible
- **WHEN** the staging issuer is selected
- **THEN** the UI SHALL display a persistent banner or badge indicating “Sandbox/Staging”


