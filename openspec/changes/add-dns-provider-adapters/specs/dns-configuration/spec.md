## ADDED Requirements

### Requirement: Cloudflare DNS adapter
The system SHALL support Cloudflare DNS-01 automation by creating and deleting TXT records via the Cloudflare API using a stored API token.

#### Scenario: Create TXT record with zone id
- **WHEN** a Cloudflare provider includes a zone id and a TXT record is requested
- **THEN** the system SHALL create the TXT record directly in that zone without additional lookups

#### Scenario: Create TXT record without zone id
- **WHEN** a Cloudflare provider omits the zone id and a TXT record is requested
- **THEN** the system SHALL resolve the zone id using the domain suffix before creating the record

### Requirement: DigitalOcean DNS adapter
The system SHALL support DigitalOcean DNS-01 automation by creating and deleting TXT records via the DigitalOcean API using a stored API token.

#### Scenario: Create TXT record
- **WHEN** a DigitalOcean provider manages `example.com` and a TXT record is requested for `_acme-challenge.example.com`
- **THEN** the system SHALL create the TXT record in the `example.com` domain

### Requirement: Route 53 DNS adapter
The system SHALL support Route 53 DNS-01 automation by creating and deleting TXT records via the AWS Route 53 API using stored credentials and a hosted zone id.

#### Scenario: Create TXT record with hosted zone id
- **WHEN** a Route 53 provider includes a hosted zone id and a TXT record is requested
- **THEN** the system SHALL create the TXT record in that hosted zone

### Requirement: Provider-specific configuration validation
The system SHALL validate required configuration fields before attempting DNS API calls.

#### Scenario: Cloudflare missing token
- **WHEN** a Cloudflare provider is created without an API token
- **THEN** the system SHALL reject the request with a validation error

#### Scenario: Route 53 missing hosted zone id
- **WHEN** a Route 53 provider is created without a hosted zone id
- **THEN** the system SHALL reject the request with a validation error

### Requirement: Provider error normalization
The system SHALL map provider API failures to a small, consistent set of error categories for UI display.

#### Scenario: Authentication failure
- **WHEN** provider credentials are invalid
- **THEN** the system SHALL return an `auth_error` category

#### Scenario: Rate limit failure
- **WHEN** provider API calls are rate limited
- **THEN** the system SHALL return a `rate_limited` category

#### Scenario: Network failure
- **WHEN** provider API calls fail due to network errors
- **THEN** the system SHALL return a `network_error` category
