## Context
DNS-01 validation is required for ACME issuance without inbound HTTP reachability. This change establishes a stable adapter interface so manual and automated DNS providers can share the same orchestration flow.

## Goals / Non-Goals
- Goals:
  - A pluggable DNS adapter interface in Rust.
  - A manual adapter that enables end-to-end issuance with no DNS API integration.
  - A propagation-check loop with clear progress and error reporting.
- Non-Goals:
  - Implementing a first automated provider (e.g., Cloudflare) — planned later.
  - Perfect DNS resolution correctness across all edge cases in v0.

## Decisions
- Decision: DNS adapter operations MUST run in Rust; UI only triggers actions and displays instructions/progress.
- Decision: DNS adapter credentials (for future providers) MUST be handled via `SecretStore` reference ids.
- Decision: Zone mapping is explicit and persisted so the system can choose adapters deterministically.

## Risks / Trade-offs
- DNS propagation is inherently flaky → Mitigation: configurable retries/timeouts and good user-facing errors.
- Multiple TXT records may exist → Mitigation: treat TXT answers as a set and confirm presence of expected value.

## Migration Plan
- Add new adapters by implementing `DnsAdapter` and registering them; manual remains as fallback.

## Open Questions
- Which DNS resolvers should be used for propagation checks (system vs authoritative vs public)?


