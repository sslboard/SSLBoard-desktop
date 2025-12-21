## Context

DNS provider configuration and test flows are now in place, but the provider adapters are stubbed. This change adds real adapters for Cloudflare, DigitalOcean, and AWS Route 53, including provider-specific configuration fields and error handling.

## Goals / Non-Goals

- Goals:
  - Implement TXT record create/delete for Cloudflare, DigitalOcean, and Route 53.
  - Validate required provider configuration (zone id or credentials) before API calls.
  - Provide consistent error messages suitable for UI display.
  - Reuse existing SecretStore for credentials; do not expose secrets to the UI.
- Non-Goals:
  - DNS zone auto-discovery (optional future enhancement).
  - Multi-account or multi-credential routing within a single provider config.
  - Support for providers beyond Cloudflare, DigitalOcean, and Route 53.

## Decisions

### Decision: Minimal provider-specific config fields
- Cloudflare: API token required, optional zone id (skip lookup if provided).
- DigitalOcean: API token required, domain (zone) derived from suffix; no extra fields.
- Route 53: Access key + secret stored in SecretStore; hosted zone id required.

### Decision: Provider adapters live in `issuance/dns_providers.rs`
Adapter logic remains in Rust core with strict secret handling. UI only supplies non-secret config values.

### Decision: Error mapping to a small stable set
Adapter errors map to:
- `auth_error` (invalid credentials / denied)
- `not_found` (zone/record missing)
- `rate_limited`
- `network_error`
- `unknown`

## Risks / Trade-offs

- API rate limits and eventual consistency can cause flaky propagation checks.
- Route 53 credentials may require additional IAM scope; failures should surface clearly.

## Migration Plan

1. Add provider config fields to DTOs and UI (Cloudflare zone id, Route 53 hosted zone id).
2. Implement adapters and wire into test flow.
3. Add targeted unit tests for request signing and DNS record path construction.

## Open Questions

- Should we add optional zone auto-discovery for Cloudflare or Route 53 as a follow-up?
