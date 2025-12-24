## ADDED Requirements
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
The Issue page SHALL present key algorithm options and size/curve choices that match the supported list.

#### Scenario: User views key options
- **WHEN** the Issue page renders the issuance form
- **THEN** it SHALL display RSA sizes 2048/3072/4096 and ECDSA curves P-256/P-384 as selectable options
