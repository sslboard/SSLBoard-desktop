## ADDED Requirements
### Requirement: Internationalized domain name handling
The system SHALL accept Unicode internationalized domain names (IDNs) from the UI, normalize and convert them to ASCII (IDNA A-labels) in the Rust core, and use the ASCII form for ACME orders and DNS-01 record generation. The system SHALL preserve the Unicode form for UI display.

#### Scenario: Unicode input is accepted and displayed
- **WHEN** a user enters a Unicode domain name for issuance
- **THEN** the UI SHALL display the Unicode form throughout the workflow
- **AND** the Rust core SHALL store and use the ASCII (IDNA) form for validation and issuance

#### Scenario: ACME order uses ASCII labels
- **WHEN** a managed issuance request includes Unicode domain names
- **THEN** the Rust core SHALL convert each label to its ASCII (IDNA) form before creating the ACME order

#### Scenario: Invalid IDN label rejected
- **WHEN** a managed issuance request includes a Unicode domain name that fails IDNA validation
- **THEN** the Rust core SHALL reject the request with a validation error before starting issuance
