## ADDED Requirements
### Requirement: Automated issuance lifecycle
The system SHALL handle the full certificate issuance lifecycle (DNS placement, propagation polling, and finalization) as an asynchronous background task initiated by the Rust core.

#### Scenario: Fully automated issuance success
- **WHEN** a managed issuance is started with an automated DNS provider
- **THEN** the system SHALL return an immediate acknowledgment to the UI
- **AND** automatically place DNS records
- **AND** poll for propagation in the background
- **AND** finalize the ACME order upon success
- **AND** emit status events to the UI throughout the process

#### Scenario: Manual DNS intervention
- **WHEN** a managed issuance is started with a "Manual" DNS provider
- **THEN** the system SHALL pause and emit a `manual-dns-required` event
- **AND** SHALL only resume polling/finalization after a user confirmation event from the UI

### Requirement: Issue page status observation
The Issue page SHALL act as a passive observer of the background issuance task, updating its state based on events received from the backend rather than driving the workflow steps manually.

#### Scenario: Issue page updates progress
- **WHEN** an issuance task is running in the background
- **THEN** the Issue page SHALL listen for `issuance-progress` events
- **AND** update the UI status (e.g., progress bar, status text) to reflect the current backend state (PROPAGATING, FINALIZING, etc.)
- **AND** SHALL NOT require the user to manually trigger "Check Propagation" or "Complete Issuance"
