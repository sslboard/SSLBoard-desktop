## MODIFIED Requirements

### Requirement: Preferences storage

The system SHALL persist user preferences in local metadata storage keyed by a preference name. Preferences MUST be stored in the unified application SQLite database (`sslboard.sqlite`) alongside other non-secret metadata.

#### Scenario: Set and retrieve preference

- **WHEN** the user sets a preference value
- **THEN** the system SHALL store it and return the same value when requested later

