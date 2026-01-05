## ADDED Requirements
### Requirement: Unified DNS provider testing
The system SHALL provide a single command to test a DNS provider that performs both credential validation and a functional end-to-end test (create, propagate, cleanup).

#### Scenario: Full provider test success
- **WHEN** a user initiates a provider test for a configured DNS provider
- **THEN** the system SHALL first validate the credentials via the provider API
- **AND** if valid, create a temporary test TXT record
- **AND** poll for its propagation
- **AND** clean up the record after verification
- **AND** return a comprehensive success result including timing metadata

