## MODIFIED Requirements
### Requirement: Timeout handling standardized
All external API calls and background polling tasks SHALL have proper timeout handling and cancellation support.

#### Scenario: DNS propagation polling respects timeouts
- **WHEN** the backend is polling for DNS propagation in an issuance task
- **THEN** timeouts SHALL be configurable
- **AND** operations SHALL be cancellable via the UI or system events
- **AND** timeout errors SHALL be properly categorized and reported to the UI

