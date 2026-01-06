# certificate-issuance Specification

## Purpose
TBD - created by archiving change add-issuance-key-options. Update Purpose after archive.
## Requirements
### Requirement: Managed issuance key options
The system SHALL support managed issuance key generation using RSA 2048/3072/4096 and ECDSA P-256/P-384.

#### Scenario: RSA 3072 is requested
- **WHEN** a managed issuance request specifies RSA with key size 3072
- **THEN** the generated private key SHALL be RSA 3072

#### Scenario: ECDSA P-384 is requested
- **WHEN** a managed issuance request specifies ECDSA with curve P-384
- **THEN** the generated private key SHALL be ECDSA P-384

### Requirement: Issuance request carries key parameters
The managed issuance request SHALL include explicit key parameters and pass them from the UI to the Rust core.

#### Scenario: UI passes RSA 4096
- **WHEN** the user selects RSA 4096 on the Issue page
- **THEN** the `start_managed_issuance` request SHALL include `key_algorithm: "rsa"` and `key_size: 4096`

#### Scenario: UI passes ECDSA P-256
- **WHEN** the user selects ECDSA P-256 on the Issue page
- **THEN** the `start_managed_issuance` request SHALL include `key_algorithm: "ecdsa"` and `key_curve: "p256"`

### Requirement: Key parameter validation and defaults
The Rust core SHALL validate key parameter combinations and SHALL default to RSA 2048 when key parameters are omitted.

#### Scenario: Missing parameters default to RSA 2048
- **WHEN** a managed issuance request does not include key parameters
- **THEN** the Rust core SHALL treat the request as RSA 2048

#### Scenario: Invalid parameters are rejected
- **WHEN** a managed issuance request includes an unsupported key size or curve
- **THEN** the Rust core SHALL reject the request with a validation error before starting issuance

### Requirement: Issue page key selection controls
The Issue page SHALL present a single combined dropdown with key algorithm options and size/curve choices that match the supported list.

#### Scenario: User views key options
- **WHEN** the Issue page renders the issuance form
- **THEN** it SHALL display RSA sizes 2048/3072/4096 and ECDSA curves P-256/P-384 as selectable options

### Requirement: Persist key algorithm metadata
The system SHALL store the selected key algorithm and size/curve in certificate metadata for display and filtering.

#### Scenario: Metadata reflects the selected key type
- **WHEN** a managed issuance completes using ECDSA P-256
- **THEN** the certificate record SHALL include key algorithm metadata indicating ECDSA P-256

### Requirement: Single key algorithm per issuance request
Each managed issuance request SHALL specify exactly one key algorithm and size/curve, and the system SHALL issue one certificate per request.

#### Scenario: RSA and ECDSA both required
- **WHEN** a user needs both RSA and ECDSA certificates for the same names
- **THEN** the system SHALL require separate managed issuance requests for each key algorithm

### Requirement: Automatic issuance workflow UI
The Issue page SHALL allow a single Start action to initiate issuance and SHALL automatically advance through DNS verification and finalization while displaying step status. If manual DNS configuration is required, the UI SHALL display the required DNS records and SHALL gate progression with a Continue Issuance action.

#### Scenario: Issuance completes successfully
- **WHEN** the user starts issuance from the Issue page
- **THEN** the UI SHALL show each step running in sequence and end with a completed certificate view

#### Scenario: Manual DNS configuration required
- **WHEN** the workflow detects manual DNS configuration is required
- **THEN** the UI SHALL display the required DNS records and wait for the user to continue issuance before resuming the workflow

#### Scenario: DNS propagation verification retries automatically
- **WHEN** DNS verification begins after the user starts issuance or continues after manual DNS setup
- **THEN** the UI SHALL automatically retry DNS propagation verification for at least one minute before surfacing a failure state

#### Scenario: Issuance fails during a non-DNS step
- **WHEN** a non-DNS step fails after the user starts issuance
- **THEN** the UI SHALL show the failing step and surface an error state with a retry action instead of a completion view

### Requirement: Completed certificate view content
The completed certificate view SHALL display the certificate common name, SANs, expiry, and key type, and SHALL provide copy and export actions.

#### Scenario: Completion view shows certificate details and actions
- **WHEN** issuance finishes successfully
- **THEN** the UI SHALL show the certificate common name, SANs, expiry, key type, and copy/export actions

### Requirement: Automated issuance completion via backend polling
The system SHALL handle DNS propagation polling and ACME finalization in the trusted Rust core during issuance completion, so the UI does not need standalone DNS propagation polling commands.

#### Scenario: Automated issuance success
- **WHEN** a managed issuance is started with an automated DNS provider
- **THEN** the system SHALL return an immediate acknowledgment to the UI
- **AND** automatically place DNS records
- **AND** upon completion, the system SHALL poll for propagation and finalize the ACME order
- **AND** store the resulting certificate in the inventory

#### Scenario: Manual DNS intervention
- **WHEN** a managed issuance is started with a "Manual" DNS provider
- **THEN** the system SHALL return DNS record instructions that indicate a manual adapter
- **AND** the UI SHALL proceed with completion only after explicit user confirmation

### Requirement: Issue page auto-advances for automated providers
The Issue page SHALL initiate issuance and, when all DNS records are automated, auto-advance to completion without requiring a separate "Check Propagation" step in the UI.

#### Scenario: Issue page completes automatically
- **WHEN** a user starts managed issuance and all DNS records are automated
- **THEN** the UI SHALL call the completion command and display success or failure
- **AND** SHALL NOT require the user to manually trigger propagation checks

