## MODIFIED Requirements

### Requirement: Supported secret kinds for v0
The system SHALL support storing secrets for at least these kinds:
- ACME account key references
- Managed private key references
- DNS provider API token references (owned by DnsProvider entities, not standalone)

#### Scenario: DNS provider token stored via provider creation flow
- **WHEN** the user creates a DNS provider with an API token
- **THEN** the system SHALL store the token as a secret with kind "dns_provider_token" and link it to the provider via `secret_ref`

#### Scenario: DNS provider token deleted with provider
- **WHEN** the user deletes a DNS provider
- **THEN** the system SHALL also delete the associated secret from SecretStore

## REMOVED Requirements

### Requirement: DNS credential ref is usable by the DNS adapter layer
**Reason**: DNS credentials are now managed through the DnsProvider entity rather than as standalone secrets. The functionality is replaced by the DNS provider configuration system.
**Migration**: Existing DNS credential secrets should be migrated to DnsProvider entities during the schema migration.
