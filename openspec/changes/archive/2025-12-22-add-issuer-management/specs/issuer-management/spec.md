## ADDED Requirements

### Requirement: Issuer records are persisted as first-class entities
The system SHALL persist issuer records in `issuance.sqlite` with a unique `issuer_id`, display label, issuer type, environment, lifecycle state (selected/disabled), timestamps, and an issuer-specific parameter payload.

#### Scenario: Issuer persists across restarts
- **WHEN** a user creates an issuer entry
- **THEN** the issuer SHALL appear in issuer listings after application restart

### Requirement: Users can add and manage issuers
The system SHALL allow users to create, update, disable, and select issuer entries via the UI without exposing secret values.

#### Scenario: User creates an issuer
- **WHEN** the user submits a new issuer configuration
- **THEN** the issuer SHALL appear in the issuer list and be selectable

#### Scenario: Disabled issuer cannot be selected
- **WHEN** an issuer is marked disabled
- **THEN** the issuer SHALL NOT be selectable for issuance

### Requirement: Built-in Let's Encrypt staging and production issuers
The system SHALL seed two ACME issuers for Let's Encrypt staging and production with standard directory URLs; staging SHALL be selected by default and production SHALL be disabled until explicitly enabled.

#### Scenario: Fresh install defaults to staging
- **WHEN** the issuer list is initialized for the first time
- **THEN** Let's Encrypt staging SHALL be selected and production SHALL be disabled

### Requirement: ACME issuers require contact email and ToS acceptance
The system SHALL require a contact email and explicit Terms of Service acceptance before registering an ACME account for an issuer, and SHALL persist the ToS acceptance state.

#### Scenario: ACME account registration is blocked without consent
- **WHEN** a user attempts to register an ACME account without an email or ToS acceptance
- **THEN** the system SHALL return a validation error and SHALL NOT proceed

#### Scenario: ToS acceptance is stored
- **WHEN** a user accepts the ACME Terms of Service
- **THEN** the system SHALL persist the acceptance state with the issuer record
