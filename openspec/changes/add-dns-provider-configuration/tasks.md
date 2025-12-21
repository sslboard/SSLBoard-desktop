## 1. Data Model & Storage

- [ ] 1.1 Define `DnsProvider` struct with embedded `domain_suffixes` in `src-tauri/src/storage/`
- [ ] 1.2 Create new SQLite schema for `dns_providers` table (id, provider_type, label, domain_suffixes JSON, secret_ref, config JSON, timestamps)
- [ ] 1.3 Implement CRUD operations for `DnsProvider` (create, list, get, update, delete)
- [ ] 1.4 Implement `resolve_provider_for_domain()` with suffix matching and ambiguity detection
- [ ] 1.5 Write migration logic from `dns_zone_mappings` to new `dns_providers` schema

## 2. Tauri Commands

- [ ] 2.1 Add `dns_provider_list` command returning all providers (with metadata, suffixes, no tokens)
- [ ] 2.2 Add `dns_provider_create` command accepting type, label, domain suffixes, and API token
- [ ] 2.3 Add `dns_provider_update` command for label, suffixes, and optionally new token
- [ ] 2.4 Add `dns_provider_delete` command (removes provider and associated secret)
- [ ] 2.5 Add `dns_provider_test` command that creates/verifies/removes a test TXT record
- [ ] 2.6 Add `dns_resolve_provider` command that returns provider for a given hostname (for preview)
- [ ] 2.7 Update `start_managed_issuance` to use new provider resolution

## 3. Test Connection Feature

- [ ] 3.1 Define test connection flow: generate random record name, create, poll for propagation, cleanup
- [ ] 3.2 Implement test TXT record creation via provider adapter
- [ ] 3.3 Implement propagation verification (reuse existing DoH lookup logic)
- [ ] 3.4 Implement test TXT record cleanup
- [ ] 3.5 Return detailed success/failure result with timing and error info

## 4. UI Components

- [ ] 4.1 Create DNS Providers settings page (`src/pages/settings/DnsProviders.tsx`)
- [ ] 4.2 Implement provider list view with type, label, domain suffixes display
- [ ] 4.3 Implement "Add Provider" dialog/form (type select, label, domain suffixes textarea, API token)
- [ ] 4.4 Implement "Edit Provider" dialog/form (update label, suffixes; optional token update)
- [ ] 4.5 Implement "Test Connection" button with progress indicator and result display
- [ ] 4.6 Implement provider delete with confirmation
- [ ] 4.7 Add overlap/ambiguity warning display when suffixes conflict across providers
- [ ] 4.8 Update sidebar navigation to include DNS Providers settings

## 5. Integration

- [ ] 5.1 Update issuance flow to use new provider resolution
- [ ] 5.2 Display resolved provider info in issuance wizard (which provider will handle DNS)
- [ ] 5.3 Show "manual DNS required" message when no automatic provider matches
- [ ] 5.4 Show ambiguity warning if multiple providers match

## 6. Cleanup

- [ ] 6.1 Remove old `dns_zone_mappings` code paths after migration
- [ ] 6.2 Update secret-store: DNS tokens now owned by providers (remove standalone DNS credential kind)
