# Change: DNS provider adapters for Cloudflare, DigitalOcean, and Route 53

## Why

Automatic DNS-01 challenges require real provider API integrations. The current DNS provider configuration and test flow exist but adapters are stubbed, so automation and connection testing cannot succeed.

## What Changes

- Add DNS adapter implementations for Cloudflare, DigitalOcean, and AWS Route 53 using official Rust SDKs.
- Implement automatic zone discovery (no user input required for zone IDs).
- Support multiple secrets per provider (Route 53 access key + secret stored separately).
- Add token validation command to verify permissions before provider creation.
- Implement structured error categories (enum-based, not string parsing).
- Add cleanup of all secrets when a provider is deleted.

## Impact

- Affected specs: `dns-configuration` (modify to add adapter behavior, zone discovery, and multi-secret support)
- Affected code (planned):
  - `src-tauri/src/issuance/dns_providers.rs` (real adapters with SDKs)
  - `src-tauri/src/core/commands.rs` (test flow, token validation, secret cleanup)
  - `src-tauri/src/core/types.rs` (error category enum, Route 53 secret types)
  - `src-tauri/src/secrets/types.rs` (Route 53 credential secret kinds)
  - `src/pages/settings/DnsProviders.tsx` (remove zone ID fields, add token test button)
  - `src/lib/dns-providers.ts` (error category types)
