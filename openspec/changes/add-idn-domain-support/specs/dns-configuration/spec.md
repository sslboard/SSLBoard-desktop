## ADDED Requirements
### Requirement: IDN-aware zone suffix matching
The system SHALL perform DNS zone suffix matching using IDNA-normalized ASCII labels so that Unicode domains match the correct provider zones.

#### Scenario: Unicode domain matches ASCII zone suffix
- **WHEN** a DNS provider zone is stored in ASCII (IDNA A-label) form
- **AND** a TXT record is requested for the equivalent Unicode domain name
- **THEN** the system SHALL match the domain to the zone after IDNA normalization

#### Scenario: Unicode zone matches Unicode domain
- **WHEN** a DNS provider zone is stored as Unicode
- **AND** a TXT record is requested for a Unicode domain name in the same IDN
- **THEN** the system SHALL match the domain to the zone after IDNA normalization
