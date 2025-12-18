## Context
Step 5 of the roadmap delivers an end-to-end issuance flow using the existing ACME staging issuer and manual DNS adapter. The flow supports locally generated keys (stored as secret refs) while keeping secrets confined to the Rust core. CSR import is explicitly deferred.

## Goals / Non-Goals
- Goals:
  - Wizard experience to collect domains, perform DNS-01, and show completion.
  - Rust orchestration that starts ACME orders, validates domain inputs, polls DNS, finalizes, and stores certificates as Managed.
  - Secret hygiene: UI never sees private keys; managed key stays in SecretStore.
- Non-Goals:
  - Automated DNS providers (covered by future adapters).
  - Production ACME issuance or issuer switching safety UX beyond existing sandbox/defaults.
  - CSR import (follow-up change).

## Decisions
- Decision: DNS-01 manual adapter remains the default; flow relies on existing zone mapping to pick adapters.
- Decision: Key generation occurs in Rust; private keys are persisted only as SecretStore refs and never round-tripped through the UI.
- Decision: CSR import is deferred; this change only supports managed key generation from user-entered domains.

## Risks / Trade-offs
- DNS propagation flakiness can delay issuance → Mitigation: polling with clear timeout/errors and retry guidance.
- Domain validation errors could block issuance → Mitigation: fail fast with actionable messages before starting orders.
- Future CSR support will need clear SAN authority rules → Mitigation: define CSR-as-authoritative semantics in a follow-up change.

## Migration Plan
- Start with manual DNS; future automated adapters can plug into the same interface without changing the wizard structure.
