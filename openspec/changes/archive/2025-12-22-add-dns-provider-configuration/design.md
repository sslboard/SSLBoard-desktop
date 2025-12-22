## Context

DNS-01 challenges require creating TXT records in the domain's DNS zone. For automation, the system needs API credentials for DNS providers. Different domains may use different providers (Cloudflare for `sslboard.com`, DigitalOcean for `other.com`), and the same provider credentials may manage multiple zones.

The current implementation has a simple `dns_zone_mappings` table with `hostname_pattern`, `adapter_id`, and `secret_ref`. This change evolves it into a unified DNS provider model where each provider configuration includes both credentials and the domain suffixes it manages.

## Goals / Non-Goals

- Goals:
  - Define a clear data model for DNS provider configurations with embedded domain suffixes
  - Support multiple providers per user with proper credential storage
  - Enable domain-to-provider resolution via suffix matching
  - Detect and warn about overlapping/ambiguous configurations
  - Keep manual DNS as fallback for domains without automatic configuration
  - UI for complete DNS provider lifecycle (add/edit/remove providers with their domains)
  - Test connection feature to validate provider credentials work
- Non-Goals:
  - Implementing actual DNS provider API integrations (Cloudflare, etc.) — deferred to separate changes
  - DNS zone auto-discovery from provider APIs
  - DNSSEC validation

## Decisions

### Decision: Single-entity data model with embedded domain suffixes

DNS configuration uses a single `DnsProvider` entity that includes both credentials and the domain suffixes it manages.

Rationale: Simpler UX and data model. When configuring a provider, you naturally specify which domains it handles. No need for a separate assignment layer.

### Decision: Domain suffixes use simple suffix matching (longest wins)

Domain suffixes use plain domain names (no wildcard syntax):

- `sslboard.com` → matches `sslboard.com` (apex) AND any subdomain (`www.sslboard.com`, `desktop.sslboard.com`, etc.)
- `api.sslboard.com` → matches `api.sslboard.com` AND deeper subdomains (`v1.api.sslboard.com`)

**The longest matching suffix wins.** For example, if provider A has `sslboard.com` and provider B has `api.sslboard.com`, a request for `api.sslboard.com` selects provider B (longer suffix = more specific). If multiple providers have the same suffix, the system warns about ambiguity.

A single provider can include multiple comma/space-separated suffixes (e.g., `sslboard.com, qcready.com`) since the same API token often manages multiple zones.

### Decision: Provider credentials stored via SecretStore

Provider API tokens are stored in the SecretStore and referenced by `secret_ref` in the DnsProvider record. The UI never sees the actual token value after initial entry.

### Decision: Provider types are an enum with provider-specific params

Supported provider types (`cloudflare`, `digitalocean`, `route53`, `manual`) are defined in code. Each provider type may require different parameters (e.g., API token for Cloudflare, Zone ID + credentials for Route 53). These are added as needed when implementing each provider adapter.

### Decision: Manual DNS is implicit fallback

If no DnsProvider matches a requested domain, the system falls back to manual DNS. Users don't need to explicitly configure manual mode.

### Decision: Test connection validates credentials

Each provider configuration includes a "Test Connection" feature that:

1. Generates a random TXT record name (e.g., `_sslboard-test-<random>.example.com`)
2. Attempts to create the TXT record via the provider API
3. Verifies the record propagates (visible via DNS lookup)
4. Removes the TXT record
5. Reports success or failure with details

This validates credentials work before the user needs them for actual issuance.

## Data Model

### DnsProvider

```
id: dns_prov_<ulid>          -- Stable identifier
provider_type: string        -- "cloudflare" | "digitalocean" | "route53" | "manual"
label: string                -- User-friendly name ("Production Cloudflare", "Personal DO")
domain_suffixes: string[]    -- ["sslboard.com", "qcready.com"] (domains this provider manages)
secret_ref: string?          -- Reference to API token in SecretStore (null for manual)
config: json?                -- Provider-specific settings (zone IDs, etc.) added as needed
created_at: datetime
updated_at: datetime
```

## Algorithm: Resolving provider for a domain

```
fn resolve_provider(hostname: &str) -> Option<DnsProvider> {
    1. Fetch all DnsProviders
    2. For each provider, check if any of its domain_suffixes match hostname
    3. Collect all matching (provider, suffix) pairs
    4. Sort by suffix length DESC (most specific first)
    5. If multiple matches with same specificity exist, flag as ambiguous
    6. Return the best match, or None (falls back to manual)
}
```

## Risks / Trade-offs

- **Embedded vs separate**: Embedding suffixes in provider is simpler but means you can't easily see "all domains" across providers. Acceptable trade-off for simpler UX.
- **Migration**: Existing `dns_zone_mappings` will be migrated to new schema.

## Migration Plan

1. Create new `dns_providers` table
2. Migrate existing `dns_zone_mappings` entries:
   - For each unique `(adapter_id, secret_ref)` pair, create a DnsProvider
   - Aggregate hostname_patterns into `domain_suffixes` array
3. Drop old `dns_zone_mappings` table (or keep for rollback safety)

## Open Questions

- None remaining (provider-specific params added as needed per provider implementation)
