## ADDED Requirements

### Requirement: Pluggable DNS adapter interface
The system SHALL provide a `DnsAdapter` interface in the trusted Rust core with operations to present a TXT record, clean up a TXT record, and check propagation status.

#### Scenario: Presenting a TXT record
- **WHEN** the DNS challenge engine requests that a TXT record be presented
- **THEN** the selected adapter SHALL attempt to create or instruct creation of the required TXT record

#### Scenario: Cleaning up a TXT record
- **WHEN** an ACME authorization completes or fails
- **THEN** the DNS challenge engine SHALL invoke the adapter cleanup operation for the TXT record

### Requirement: Manual DNS adapter
The system SHALL provide a `ManualDnsAdapter` that returns user-facing instructions (record name, value, and zone context) rather than calling an external DNS API.

#### Scenario: Manual instructions are shown to the user
- **WHEN** the manual adapter is selected for a DNS-01 challenge
- **THEN** the system SHALL provide the UI the exact TXT record name and value to create

### Requirement: Propagation check loop
The system SHALL provide a propagation-check loop that can be triggered by the UI after the user claims the TXT record has been added, and it SHALL return progress and failure reasons.

#### Scenario: Propagation succeeds
- **WHEN** the expected TXT value becomes visible via DNS lookup
- **THEN** the system SHALL report success and allow issuance to continue

#### Scenario: Propagation times out
- **WHEN** the expected TXT value does not appear within the configured retry/timeout budget
- **THEN** the system SHALL return a timeout error and guidance suitable for user display

### Requirement: Zone mapping for adapter selection
The system SHALL support a persisted mapping from hostname to DNS zone to adapter configuration so the correct adapter can be selected for a given domain.

#### Scenario: Hostname maps to zone and adapter
- **WHEN** the user begins issuance for `foo.example.com`
- **THEN** the system SHALL resolve `example.com` as the zone (if configured) and select its configured adapter

### Requirement: DNS-01 stepper UI
The UI SHALL provide a DNS-01 stepper experience that displays the exact `_acme-challenge` TXT record and allows the user to trigger propagation checks, including progress and common failure states (NXDOMAIN, wrong TXT value, TTL delays).

#### Scenario: User triggers propagation checks
- **WHEN** the user clicks “I’ve added it”
- **THEN** the UI SHALL request propagation checks and display updated status until success or failure


