## ADDED Requirements
### Requirement: Optional ACME profile selection
When a CA advertises ACME profiles, the system SHALL allow the user to select one and the Rust core SHALL validate and apply it to the issuance request.

#### Scenario: Profiles are advertised by the CA
- **WHEN** the ACME directory metadata includes one or more profiles
- **THEN** the Issue workflow SHALL display those profiles as selectable options

#### Scenario: User selects a valid profile
- **WHEN** a user selects a profile that is advertised by the CA
- **THEN** the Rust core SHALL include that profile in the ACME newOrder request

#### Scenario: Profiles are not advertised
- **WHEN** the ACME directory metadata does not include profiles
- **THEN** the Issue workflow SHALL hide profile selection and the Rust core SHALL omit the profile from newOrder

#### Scenario: User selects an invalid profile
- **WHEN** a managed issuance request includes a profile not present in the CA metadata
- **THEN** the Rust core SHALL reject the request with a validation error before starting issuance

### Requirement: Persist selected ACME profile
The system SHALL store the selected ACME profile in certificate metadata when one was applied during issuance.

#### Scenario: Metadata includes the selected profile
- **WHEN** a managed issuance completes with a selected profile
- **THEN** the certificate record SHALL include the profile identifier for display and filtering
