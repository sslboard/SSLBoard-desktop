## ADDED Requirements
### Requirement: Unified DNS provider testing
The system SHALL provide a single command to test a DNS provider that performs a functional end-to-end test (create, propagate, cleanup). Credential issues (auth/missing permissions) SHALL be surfaced as part of the same test result during the create stage.

#### Scenario: Full provider test success
- **WHEN** a user initiates a provider test for a configured DNS provider
- **THEN** the system SHALL create a temporary test TXT record
- **AND** poll for its propagation
- **AND** clean up the record after verification
- **AND** return a comprehensive success result including timing metadata

#### Scenario: Credential failure is reported as part of the test

- **WHEN** a user initiates a provider test with invalid credentials
- **THEN** the test SHALL fail at the create stage
- **AND** return an error category appropriate to authentication/authorization failures
