## ADDED Requirements
### Requirement: ACME client migration preserves issuance behavior
The system SHALL preserve existing ACME issuance behavior and response shapes when migrating to a new ACME client library.

#### Scenario: Managed issuance completes after client migration
- **WHEN** the user issues a certificate from DNS names using an existing issuer
- **THEN** the issuance flow SHALL complete with the same DNS instruction and completion responses as before the migration
