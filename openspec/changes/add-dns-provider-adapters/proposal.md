# Change: DNS provider adapters for Cloudflare, DigitalOcean, and Route 53

## Why
Automatic DNS-01 challenges require real provider API integrations. The current DNS provider configuration and test flow exist but adapters are stubbed, so automation and connection testing cannot succeed.

## What Changes
- Add DNS adapter implementations for Cloudflare, DigitalOcean, and AWS Route 53.
- Introduce provider-specific configuration fields (e.g., Route 53 hosted zone id; optional Cloudflare zone id).
- Implement create/update/delete TXT operations for each provider adapter and wire into the test-connection flow.
- Add validation for required provider configuration values and error mapping for clearer UI feedback.

## Impact
- Affected specs: `dns-configuration` (modify to add adapter behavior and config fields)
- Affected code (planned):
  - `src-tauri/src/issuance/dns_providers.rs` (real adapters)
  - `src-tauri/src/core/commands.rs` (test flow uses adapters)
  - `src/pages/settings/DnsProviders.tsx` (provider-specific fields)
  - `src/lib/dns-providers.ts` (DTO additions)
