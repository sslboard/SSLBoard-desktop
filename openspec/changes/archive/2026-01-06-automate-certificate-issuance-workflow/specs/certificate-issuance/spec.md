## ADDED Requirements
### Requirement: Automated issuance completion via backend polling
The system SHALL handle DNS propagation polling and ACME finalization in the trusted Rust core during issuance completion, so the UI does not need standalone DNS propagation polling commands.

#### Scenario: Automated issuance success
- **WHEN** a managed issuance is started with an automated DNS provider
- **THEN** the system SHALL return an immediate acknowledgment to the UI
- **AND** automatically place DNS records
- **AND** upon completion, the system SHALL poll for propagation and finalize the ACME order
- **AND** store the resulting certificate in the inventory

#### Scenario: Manual DNS intervention
- **WHEN** a managed issuance is started with a "Manual" DNS provider
- **THEN** the system SHALL return DNS record instructions that indicate a manual adapter
- **AND** the UI SHALL proceed with completion only after explicit user confirmation

### Requirement: Issue page auto-advances for automated providers
The Issue page SHALL initiate issuance and, when all DNS records are automated, auto-advance to completion without requiring a separate "Check Propagation" step in the UI.

#### Scenario: Issue page completes automatically
- **WHEN** a user starts managed issuance and all DNS records are automated
- **THEN** the UI SHALL call the completion command and display success or failure
- **AND** SHALL NOT require the user to manually trigger propagation checks
