# Change: Update DNS provider TXT record handling

## Why

DNS providers can return duplicate-record errors and accept unquoted TXT values with warnings. The adapters should upsert TXT records, verify edits are live, and normalize TXT content to avoid provider warnings and inconsistent behavior.

## What Changes

- Update DigitalOcean and Route 53 DNS adapters to upsert TXT records when an identical name already exists.
- Verify TXT record content via provider read-after-write before returning success.
- Normalize TXT record content to quoted string format on create/update.

## Impact

- Affected specs: `dns-configuration` (modify adapter requirements)
- Affected code:
  - `src-tauri/src/issuance/dns_providers/digitalocean.rs`
  - `src-tauri/src/issuance/dns_providers/route53.rs`
  - `src-tauri/src/issuance/dns_providers/mod.rs` (shared helpers if needed)
