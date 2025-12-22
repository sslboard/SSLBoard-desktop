# Change: Add DNS provider integration tests

## Why

The DNS provider adapters (Cloudflare, DigitalOcean, Route 53) have several implementation uncertainties that could cause production failures:

- Record name format requirements (relative vs absolute, trailing dots)
- TXT value quoting behavior (required vs automatic)
- API response format variations (quoted values, name normalization)
- Upsert behavior correctness
- Verification logic reliability

Manual testing is time-consuming and doesn't catch regressions. Integration tests against real provider APIs will validate assumptions, document actual API behavior, and catch bugs before production deployment.

## What Changes

- Add integration test framework for DNS provider adapters with feature flag gating
- Create test suite that validates record name formats, TXT value handling, upsert behavior, and verification logic for each provider
- Add test utilities for credential management, test domain setup, and cleanup
- Document actual API behavior discovered through testing
- Add CI configuration to run integration tests conditionally (e.g., on schedule or manual trigger)

## Impact

- Affected specs: `dns-configuration` (add integration test requirements)
- Affected code:
  - `src-tauri/src/issuance/dns_providers/cloudflare.rs`
  - `src-tauri/src/issuance/dns_providers/digitalocean.rs`
  - `src-tauri/src/issuance/dns_providers/route53.rs`
  - New: `src-tauri/tests/integration/dns_providers/` (test modules)
  - New: `src-tauri/tests/integration/dns_providers/test_utils.rs` (shared utilities)
  - `Cargo.toml` (add test dependencies and feature flags)

