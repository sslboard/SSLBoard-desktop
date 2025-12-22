## ADDED Requirements

### Requirement: Cloudflare DNS adapter

The system SHALL support Cloudflare DNS-01 automation by creating and deleting TXT records via the Cloudflare API using a stored API token. The system SHALL automatically discover zone IDs by listing available zones.

#### Scenario: Create TXT record with automatic zone discovery

- **WHEN** a Cloudflare provider is configured and a TXT record is requested for a domain
- **THEN** the system SHALL list available zones, match the domain suffix to a zone, and create the TXT record in that zone

#### Scenario: Token validation

- **WHEN** a user provides a Cloudflare API token
- **THEN** the system SHALL provide a test mechanism that verifies the token can list zones

### Requirement: DigitalOcean DNS adapter

The system SHALL support DigitalOcean DNS-01 automation by creating and deleting TXT records via the DigitalOcean API using a stored API token.

#### Scenario: Create TXT record

- **WHEN** a DigitalOcean provider manages `example.com` and a TXT record is requested for `_acme-challenge.example.com`
- **THEN** the system SHALL create the TXT record in the `example.com` domain

### Requirement: Route 53 DNS adapter

The system SHALL support Route 53 DNS-01 automation by creating and deleting TXT records via the AWS Route 53 API using stored access key and secret. The system SHALL automatically discover hosted zone IDs by listing available hosted zones.

#### Scenario: Create TXT record with automatic zone discovery

- **WHEN** a Route 53 provider is configured and a TXT record is requested for a domain
- **THEN** the system SHALL list available hosted zones, match the domain suffix to a hosted zone, and create the TXT record in that zone

#### Scenario: Multiple secrets for Route 53

- **WHEN** a Route 53 provider is created
- **THEN** the system SHALL store the access key and secret as separate secret references

### Requirement: Provider-specific configuration validation

The system SHALL validate required provider configuration fields before attempting DNS API calls.

#### Scenario: Cloudflare missing token

- **WHEN** a Cloudflare provider is created without an API token
- **THEN** the system SHALL reject the request with a validation error

#### Scenario: Route 53 missing credentials

- **WHEN** a Route 53 provider is created without an access key or secret
- **THEN** the system SHALL reject the request with a validation error

### Requirement: Provider error normalization

The system SHALL map provider API failures to a structured error category enum for UI display.

#### Scenario: Authentication failure

- **WHEN** provider credentials are invalid
- **THEN** the system SHALL return an `auth_error` category

#### Scenario: Rate limit failure

- **WHEN** provider API calls are rate limited
- **THEN** the system SHALL return a `rate_limited` category

#### Scenario: Network failure

- **WHEN** provider API calls fail due to network errors
- **THEN** the system SHALL return a `network_error` category

#### Scenario: Not found

- **WHEN** a zone or record is not found
- **THEN** the system SHALL return a `not_found` category

### Requirement: Secret cleanup on provider deletion

The system SHALL automatically remove all secret references associated with a DNS provider when the provider is deleted.

#### Scenario: Delete provider with multiple secrets

- **WHEN** a Route 53 provider with access key and secret secrets is deleted
- **THEN** the system SHALL delete both secret references from the secret store
