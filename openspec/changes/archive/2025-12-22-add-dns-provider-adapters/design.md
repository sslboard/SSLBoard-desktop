## Context

DNS provider configuration and test flows are now in place, but the provider adapters are stubbed. This change adds real adapters for Cloudflare, DigitalOcean, and AWS Route 53, using official Rust SDKs and automatic zone discovery.

## Goals / Non-Goals

- Goals:
  - Implement TXT record create/delete for Cloudflare, DigitalOcean, and Route 53 using official Rust SDKs.
  - Automatically discover zone IDs by listing available zones (no user input required).
  - Validate token/credential permissions before use.
  - Provide structured error categories for UI display.
  - Support multiple secrets per provider (e.g., Route 53 access key + secret).
  - Cleanup all secrets when a provider is deleted.
- Non-Goals:
  - Multi-account or multi-credential routing within a single provider config.
  - Support for providers beyond Cloudflare, DigitalOcean, and Route 53.
  - Manual zone ID entry by users.

## Decisions

### Decision: Automatic zone discovery

- Cloudflare: List zones via API, match domain suffix to zone name.
- Route 53: List hosted zones via API, match domain suffix to hosted zone name.
- DigitalOcean: Domain derived from suffix; no zone discovery needed.

### Decision: Use official Rust SDKs

- Cloudflare: Use `cloudflare` crate or direct HTTP with reqwest.
- DigitalOcean: Use `digitalocean` crate or direct HTTP with reqwest.
- Route 53: Use `aws-sdk-route53` with `aws-config`.

### Decision: Multiple secrets per provider

- Route 53: Store access key and secret as separate `SecretKind::DnsProviderToken` entries.
- Cloudflare/DigitalOcean: Single API token secret.
- Cleanup: Delete all secrets referenced by a provider when provider is deleted.

### Decision: Provider adapters live in `issuance/dns_providers.rs`

Adapter logic remains in Rust core with strict secret handling. UI only supplies non-secret config values.

### Decision: Structured error categories

Adapter errors map to an enum:

- `auth_error` (invalid credentials / denied)
- `not_found` (zone/record missing)
- `rate_limited`
- `network_error`
- `unknown`

### Decision: Token validation

Add a test command that validates tokens can list zones before allowing provider creation/update.

## Risks / Trade-offs

- API rate limits and eventual consistency can cause flaky propagation checks.
- Zone discovery adds an extra API call per operation; cache zone IDs in provider config after first discovery.
- Route 53 credentials may require additional IAM scope; failures should surface clearly via error categories.

## Migration Plan

1. Add SDK dependencies to Cargo.toml.
2. Update SecretKind to support Route 53 credentials separately.
3. Add error category enum to DTOs.
4. Implement adapters with zone discovery.
5. Add token validation command.
6. Update provider delete to cleanup multiple secrets.
7. Update UI to remove zone ID fields and add token test button.
8. Add targeted unit tests for zone discovery and error mapping.

## Open Questions

- Should we cache discovered zone IDs in provider config to avoid repeated lookups?
