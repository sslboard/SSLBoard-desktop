## Context
Step 5 of the roadmap delivers an end-to-end issuance flow using the existing ACME staging issuer and manual DNS adapter. The flow must support both locally generated keys (stored as secret refs) and CSR import while keeping secrets confined to the Rust core.

## Goals / Non-Goals
- Goals:
  - Wizard experience to collect domains, choose key mode, perform DNS-01, and show completion.
  - Rust orchestration that starts ACME orders, validates CSR inputs, polls DNS, finalizes, and stores certificates as Managed.
  - Secret hygiene: UI never sees private keys; CSR imports remain keyless.
- Non-Goals:
  - Automated DNS providers (covered by future adapters).
  - Production ACME issuance or issuer switching safety UX beyond existing sandbox/defaults.

## Decisions
- Decision: DNS-01 manual adapter remains the default; flow relies on existing zone mapping to pick adapters.
- Decision: Key generation occurs in Rust; private keys are persisted only as SecretStore refs and never round-tripped through the UI.
- Decision: CSR import path uses CSR-derived SANs as the order source of truth; mismatches are rejected with a clear error.

## Risks / Trade-offs
- DNS propagation flakiness can delay issuance → Mitigation: polling with clear timeout/errors and retry guidance.
- CSR parsing/validation errors could block issuance → Mitigation: fail fast with actionable messages before starting orders.
- Mixed SAN sets between user input and CSR → Mitigation: enforce CSR SANs as authoritative for CSR path.

## Migration Plan
- Start with manual DNS; future automated adapters can plug into the same interface without changing the wizard structure.
