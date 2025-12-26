## ADDED Requirements
### Requirement: Automatic issuance workflow UI
The Issue page SHALL allow a single Start action to initiate issuance and SHALL automatically advance through DNS verification and finalization while displaying step status. If manual DNS configuration is required, the UI SHALL display the required DNS records and SHALL gate progression with a Continue Issuance action.

#### Scenario: Issuance completes successfully
- **WHEN** the user starts issuance from the Issue page
- **THEN** the UI SHALL show each step running in sequence and end with a completed certificate view

#### Scenario: Manual DNS configuration required
- **WHEN** the workflow detects manual DNS configuration is required
- **THEN** the UI SHALL display the required DNS records and wait for the user to continue issuance before resuming the workflow

#### Scenario: DNS propagation verification retries automatically
- **WHEN** DNS verification begins after the user starts issuance or continues after manual DNS setup
- **THEN** the UI SHALL automatically retry DNS propagation verification for at least one minute before surfacing a failure state

#### Scenario: Issuance fails during a non-DNS step
- **WHEN** a non-DNS step fails after the user starts issuance
- **THEN** the UI SHALL show the failing step and surface an error state with a retry action instead of a completion view

### Requirement: Completed certificate view content
The completed certificate view SHALL display the certificate common name, SANs, expiry, and key type, and SHALL provide copy and export actions.

#### Scenario: Completion view shows certificate details and actions
- **WHEN** issuance finishes successfully
- **THEN** the UI SHALL show the certificate common name, SANs, expiry, key type, and copy/export actions
