## MODIFIED Requirements
### Requirement: Timeout handling standardized
All external API calls and polling tasks SHALL have proper timeout handling.

#### Scenario: DNS propagation polling respects timeouts
- **WHEN** the backend is polling for DNS propagation during issuance completion
- **THEN** the polling SHALL use a bounded timeout and interval
- **AND** timeout errors SHALL be properly categorized and reported to the UI
