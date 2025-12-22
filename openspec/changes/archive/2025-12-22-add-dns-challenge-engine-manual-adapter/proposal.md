# Change: DNS-01 challenge engine + pluggable DNS adapters (manual first)

## Why
DNS-01 is the core validation method for ACME issuance in this product. A manual adapter enables end-to-end issuance before any DNS provider API integrations exist, while keeping room for automated adapters later.

## What Changes
- Define a `DnsAdapter` interface in Rust for presenting and cleaning up TXT records and checking propagation.
- Implement a `ManualDnsAdapter` that returns TXT record instructions and supports a wait/recheck propagation loop.
- Add a zone mapping model (hostname → zone → adapter config) to select the appropriate adapter.
- Add a UI DNS stepper for DNS-01 that shows record instructions and drives propagation checks.

## Impact
- Affected specs: `dns-challenges`
- Affected code (planned): `src-tauri/src/issuance/dns/`, `src-tauri/src/core/commands.rs`, `src-tauri/src/core/types.rs`, `src/` issuance wizard UI


