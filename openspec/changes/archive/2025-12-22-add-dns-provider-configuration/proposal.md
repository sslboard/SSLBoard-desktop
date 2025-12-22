# Change: DNS provider configuration for automatic ACME challenges

## Why
Users need to configure DNS provider credentials (Cloudflare, DigitalOcean, Route 53, etc.) so the system can automatically add ACME DNS-01 challenges for their domains. Currently, DNS tokens can be stored in secrets but there's no structured configuration linking providers to domains. This change introduces a proper DNS provider model where each provider includes both credentials and the domain suffixes it manages.

## What Changes
- **BREAKING**: Remove the generic "DNS credential reference" secret type; replace with structured DNS provider configurations.
- Add a `DnsProvider` entity representing a configured DNS provider (type, label, API credentials, and domain suffixes it manages).
- Domain suffixes use simple suffix matching (e.g., `sslboard.com` matches apex and all subdomains).
- Support multiple domain suffixes per provider via comma/space-separated input.
- Support multiple DNS providers per user, each managing different domains.
- Provider resolution: when requesting a certificate for `desktop.sslboard.com`, find the provider whose suffix matches (most specific wins).
- Warn/notify when overlapping suffixes exist across providers (ambiguous configuration).
- Add "Test Connection" feature that creates/verifies/removes a test TXT record to validate credentials.
- Manual DNS remains as the fallback when no provider matches a domain.
- Add UI for managing DNS provider configurations (Settings → DNS Providers).

## Impact
- Affected specs: `dns-configuration` (new), `secret-store` (modify to remove standalone DNS credential kind)
- Affected code (planned): 
  - `src-tauri/src/storage/dns.rs` (refactor from zone mapping to provider model)
  - `src-tauri/src/core/commands.rs` (new commands for provider CRUD + test)
  - `src-tauri/src/core/types.rs` (new DTOs)
  - `src/` DNS settings UI
