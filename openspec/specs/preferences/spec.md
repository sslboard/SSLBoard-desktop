# preferences Specification

## Purpose
TBD - created by archiving change add-preferences-store. Update Purpose after archive.
## Requirements
### Requirement: Preferences storage
The system SHALL persist user preferences in local metadata storage keyed by a preference name.

#### Scenario: Set and retrieve preference
- **WHEN** the user sets a preference value
- **THEN** the system SHALL store it and return the same value when requested later

### Requirement: Export destination preference
The system SHALL persist the last chosen certificate export destination and use it to prefill subsequent exports.

#### Scenario: Save export destination
- **WHEN** the user selects or completes an export with a destination folder
- **THEN** the system SHALL save that destination as the export preference

#### Scenario: Prefill export destination from preference
- **WHEN** the user opens the export modal and a saved destination exists
- **THEN** the UI SHALL prefill the destination with the saved value

#### Scenario: Default export destination when preference is missing
- **WHEN** the user opens the export modal and no saved destination exists
- **THEN** the UI SHALL default the destination to the user’s Downloads folder

