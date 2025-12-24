## ADDED Requirements

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
