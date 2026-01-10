## ADDED Requirements
### Requirement: CSR-based issuance
The system SHALL allow issuance requests to supply a CSR file and SHALL derive the issuance identifiers (common name and SANs) from the CSR instead of UI-provided DNS names.

#### Scenario: CSR import is used for issuance
- **WHEN** the user selects a CSR file and starts issuance
- **THEN** the request SHALL include a CSR file reference
- **AND** the Rust core SHALL parse the CSR and use its identifiers for issuance

#### Scenario: CSR parsing fails
- **WHEN** the CSR file is malformed or has an invalid signature
- **THEN** the Rust core SHALL reject the request with a validation error before issuance begins

### Requirement: CSR validation in the Rust core
The Rust core SHALL validate the CSR signature, key algorithm, and SAN presence before proceeding with issuance, and SHALL allow CN-only CSRs with a warning.

#### Scenario: CSR missing SANs
- **WHEN** the CSR does not include any SAN entries
- **THEN** the Rust core SHALL proceed with a warning indicating SANs are missing and CN-only issuance will be used

#### Scenario: CSR key algorithm is unsupported
- **WHEN** the CSR key algorithm is not in the supported RSA or ECDSA lists
- **THEN** the Rust core SHALL return a validation error indicating the algorithm is unsupported

### Requirement: CSR creation with managed keys
The system SHALL support creating a CSR file by generating a managed private key in the Rust core, building the CSR with user-provided subject and SANs, and writing the CSR PEM to a user-selected path.

#### Scenario: User creates a CSR file
- **WHEN** the user provides a subject, SANs, and key parameters and selects a file path
- **THEN** the Rust core SHALL generate the private key, create the CSR, and write the CSR PEM to the selected path

#### Scenario: CSR creation uses supported key parameters
- **WHEN** the user selects ECDSA P-384 for CSR creation
- **THEN** the generated key and CSR SHALL use ECDSA P-384

### Requirement: Issue page CSR workflows
The Issue page SHALL let the user choose between DNS-name issuance and CSR-based issuance, and SHALL hide manual DNS name entry when a CSR is selected.

#### Scenario: CSR issuance mode is selected
- **WHEN** the user switches the Issue page to CSR issuance
- **THEN** the manual DNS name inputs SHALL be hidden
- **AND** the UI SHALL show the CSR file selection state

### Requirement: CSR metadata persistence
The system SHALL store CSR metadata on the resulting certificate record, including subject, SANs, key algorithm, and CSR source (imported or generated).

#### Scenario: CSR-derived metadata is stored
- **WHEN** issuance completes using a CSR
- **THEN** the certificate record SHALL include CSR subject, SANs, key algorithm, and source metadata
