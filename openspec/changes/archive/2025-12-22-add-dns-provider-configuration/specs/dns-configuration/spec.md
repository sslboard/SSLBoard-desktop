## ADDED Requirements

### Requirement: DNS provider entity with embedded domain suffixes
The system SHALL support a `DnsProvider` entity representing a configured DNS provider with a unique identifier, provider type, user-friendly label, list of domain suffixes it manages, and optional reference to API credentials stored in the SecretStore.

#### Scenario: Creating a Cloudflare provider with domains
- **WHEN** the user creates a DNS provider with type "cloudflare", label "Production CF", domain suffixes "sslboard.com, qcready.com", and provides an API token
- **THEN** the system SHALL store the API token in SecretStore and create a DnsProvider record with a stable `dns_prov_` prefixed identifier and the parsed domain suffixes

#### Scenario: Listing DNS providers
- **WHEN** the UI requests the list of DNS providers
- **THEN** the system SHALL return provider metadata (id, type, label, domain_suffixes, created_at) without exposing API tokens

#### Scenario: Comma or space separated suffix input
- **WHEN** the user enters "sslboard.com, qcready.com" or "sslboard.com qcready.com" in the domain suffixes field
- **THEN** the system SHALL parse and store them as separate suffixes in the provider record

### Requirement: Provider resolution by domain suffix (longest wins)
The system SHALL resolve the appropriate DNS provider for a hostname using suffix matching (no wildcard syntax), where the longest matching suffix wins.

#### Scenario: Longest suffix wins
- **WHEN** provider A has suffix "sslboard.com" (12 chars) and provider B has suffix "api.sslboard.com" (16 chars), and the user requests a certificate for "api.sslboard.com"
- **THEN** the system SHALL select provider B because it has the longest matching suffix

#### Scenario: Zone-level match for subdomain
- **WHEN** a provider has suffix "sslboard.com" and the user requests a certificate for "desktop.sslboard.com"
- **THEN** the system SHALL select that provider via suffix matching

#### Scenario: Zone-level match for apex
- **WHEN** a provider has suffix "sslboard.com" and the user requests a certificate for "sslboard.com" (apex)
- **THEN** the system SHALL select that provider (suffix matches itself)

### Requirement: Ambiguous provider detection
The system SHALL detect when multiple providers match a domain with equal specificity and warn the user about ambiguous configuration.

#### Scenario: Overlapping suffixes warning
- **WHEN** provider A (Cloudflare) and provider B (DigitalOcean) both have suffix "sslboard.com"
- **THEN** the system SHALL warn that the configuration is ambiguous and indicate which provider will be used

#### Scenario: Ambiguity flagged during issuance
- **WHEN** the user starts issuance for a domain with ambiguous provider matching
- **THEN** the system SHALL proceed with one provider but display a warning about the ambiguity

### Requirement: Manual DNS fallback
The system SHALL fall back to manual DNS challenge handling when no automatic DNS provider is configured for a requested domain.

#### Scenario: No provider configured
- **WHEN** the user requests a certificate for "unassigned.example.com" and no provider's suffixes match
- **THEN** the system SHALL use manual DNS mode, displaying TXT record instructions for the user to add manually

### Requirement: DNS provider CRUD operations
The system SHALL provide Tauri commands to create, list, update, and delete DNS providers from the UI.

#### Scenario: Creating a provider via IPC
- **WHEN** the UI sends a `dns_provider_create` command with type, label, domain suffixes, and API token
- **THEN** the system SHALL store the token in SecretStore, create the provider record, and return only the provider metadata (not the token)

#### Scenario: Updating a provider
- **WHEN** the UI sends a `dns_provider_update` command with updated label or domain suffixes
- **THEN** the system SHALL update the provider without requiring the API token to be re-entered

#### Scenario: Updating a provider's API token
- **WHEN** the UI sends a `dns_provider_update` command with a new API token value
- **THEN** the system SHALL replace the secret in SecretStore while keeping the same secret_ref

#### Scenario: Deleting a provider
- **WHEN** the UI deletes a DNS provider
- **THEN** the system SHALL remove the provider record and its associated secret from SecretStore

### Requirement: Test connection for DNS providers
The system SHALL provide a test connection feature that validates provider credentials by creating, verifying, and removing a test TXT record.

#### Scenario: Test connection succeeds
- **WHEN** the user clicks "Test Connection" for a configured provider
- **THEN** the system SHALL create a random TXT record (e.g., `_sslboard-test-<random>.<domain>`), verify it propagates via DNS lookup, remove the record, and report success

#### Scenario: Test connection fails on create
- **WHEN** the test connection cannot create the TXT record (invalid credentials, permissions, etc.)
- **THEN** the system SHALL report failure with a descriptive error message

#### Scenario: Test connection fails on propagation
- **WHEN** the TXT record is created but not visible after timeout
- **THEN** the system SHALL report failure, attempt cleanup, and suggest possible causes (propagation delay, wrong zone)

#### Scenario: Test connection cleanup
- **WHEN** the test completes (success or failure)
- **THEN** the system SHALL attempt to remove the test TXT record to avoid leaving DNS pollution

### Requirement: DNS providers settings UI
The UI SHALL provide a "Settings → DNS Providers" view for managing DNS provider configurations with their embedded domain suffixes.

#### Scenario: Viewing provider list
- **WHEN** the user navigates to DNS Providers settings
- **THEN** the UI SHALL display all configured providers with their type, label, and domain suffixes

#### Scenario: Adding a new provider
- **WHEN** the user clicks "Add Provider" and fills in type, label, domain suffixes, and API token
- **THEN** the UI SHALL call the create command and display the new provider in the list

#### Scenario: Editing a provider
- **WHEN** the user edits a provider's label or domain suffixes
- **THEN** the UI SHALL update the provider without requiring re-entry of the API token

#### Scenario: Testing a provider
- **WHEN** the user clicks "Test Connection" on a provider
- **THEN** the UI SHALL show a progress indicator, then display success or failure with details

### Requirement: Supported DNS provider types
The system SHALL support these DNS provider types: `cloudflare`, `digitalocean`, `route53`, and `manual`, with the type defined as an enum to ensure type safety.

#### Scenario: Selecting provider type during creation
- **WHEN** the user creates a new DNS provider
- **THEN** the UI SHALL present a dropdown with supported provider types

#### Scenario: Invalid provider type rejected
- **WHEN** an API request specifies an unsupported provider type
- **THEN** the system SHALL return a validation error

### Requirement: Provider-specific configuration parameters
The system SHALL support provider-specific configuration parameters beyond the API token, added as needed for each provider type implementation.

#### Scenario: Provider with extra config
- **WHEN** a provider type requires additional configuration (e.g., AWS Zone ID)
- **THEN** the system SHALL store these in a provider-specific config field and the UI SHALL display appropriate input fields
