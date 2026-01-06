# dns-configuration Specification

## Purpose
TBD - created by archiving change update-dns-provider-txt-upsert. Update Purpose after archive.
## Requirements
### Requirement: Cloudflare DNS adapter

The system SHALL support Cloudflare DNS-01 automation by creating, updating, and deleting TXT records via the Cloudflare API using a stored API token. The system SHALL automatically discover zone IDs by listing available zones. The system SHALL update an existing TXT record when a duplicate record already exists. The system SHALL verify the TXT content via a read-after-write check before returning success.

#### Scenario: Create or update TXT record with automatic zone discovery
- **WHEN** a Cloudflare provider is configured and a TXT record is requested for a domain
- **THEN** the system SHALL list available zones, match the domain suffix to a zone, and create or update the TXT record in that zone

#### Scenario: Duplicate TXT record updated
- **WHEN** a Cloudflare TXT record already exists for the requested name
- **THEN** the system SHALL update the existing record with the new value instead of failing

#### Scenario: Post-write verification
- **WHEN** the Cloudflare adapter writes a TXT record
- **THEN** the system SHALL fetch the record and confirm the stored content matches the requested value before returning success

### Requirement: DigitalOcean DNS adapter

The system SHALL support DigitalOcean DNS-01 automation by creating, updating, and deleting TXT records via the DigitalOcean API using a stored API token. The system SHALL update an existing TXT record when a duplicate record already exists. The system SHALL verify the TXT content via a read-after-write check before returning success.

#### Scenario: Create TXT record
- **WHEN** a DigitalOcean provider manages `example.com` and a TXT record is requested for `_acme-challenge.example.com`
- **THEN** the system SHALL create or update the TXT record in the `example.com` domain

#### Scenario: Duplicate TXT record updated
- **WHEN** a DigitalOcean TXT record already exists for the requested name
- **THEN** the system SHALL update the existing record with the new value instead of failing

#### Scenario: Post-write verification
- **WHEN** the DigitalOcean adapter writes a TXT record
- **THEN** the system SHALL fetch the record and confirm the stored content matches the requested value before returning success

### Requirement: Route 53 DNS adapter

The system SHALL support Route 53 DNS-01 automation by creating, updating, and deleting TXT records via the AWS Route 53 API using stored access key and secret. The system SHALL automatically discover hosted zone IDs by listing available hosted zones. The system SHALL update an existing TXT record when a duplicate record already exists. The system SHALL verify the TXT content via a read-after-write check before returning success.

#### Scenario: Create TXT record with automatic zone discovery
- **WHEN** a Route 53 provider is configured and a TXT record is requested for a domain
- **THEN** the system SHALL list available hosted zones, match the domain suffix to a hosted zone, and create or update the TXT record in that zone

#### Scenario: Duplicate TXT record updated
- **WHEN** a Route 53 TXT record already exists for the requested name
- **THEN** the system SHALL update the existing record with the new value instead of failing

#### Scenario: Post-write verification
- **WHEN** the Route 53 adapter writes a TXT record
- **THEN** the system SHALL fetch the record and confirm the stored content matches the requested value before returning success

### Requirement: Unified DNS provider testing
The system SHALL provide a single command to test a DNS provider that performs a functional end-to-end test (create, propagate, cleanup). Credential issues (auth/missing permissions) SHALL be surfaced as part of the same test result during the create stage.

#### Scenario: Full provider test success
- **WHEN** a user initiates a provider test for a configured DNS provider
- **THEN** the system SHALL create a temporary test TXT record
- **AND** poll for its propagation
- **AND** clean up the record after verification
- **AND** return a comprehensive success result including timing metadata

#### Scenario: Credential failure is reported as part of the test

- **WHEN** a user initiates a provider test with invalid credentials
- **THEN** the test SHALL fail at the create stage
- **AND** return an error category appropriate to authentication/authorization failures

