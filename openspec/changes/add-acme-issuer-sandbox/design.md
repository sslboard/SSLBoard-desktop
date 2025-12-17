## Context
Issuance must be modular and safe by default. This change introduces an issuer abstraction and the first concrete issuer: ACME against Let’s Encrypt staging.

## Goals / Non-Goals
- Goals:
  - Stable issuer interface for later issuers (LE prod, private CA, other ACME CAs).
  - Staging-first defaults with obvious UI signaling.
  - Account key storage via `SecretStore` reference ids (no secret IPC).
- Non-Goals:
  - Full issuance wizard and DNS orchestration (handled in later steps).
  - Production issuance safety switches (explicit opt-in can be added later).

## Decisions
- Decision: Default issuer MUST be Let’s Encrypt staging until the user explicitly opts into production endpoints.
- Decision: ACME account private keys MUST be stored in `SecretStore`; only references are persisted and shared with the UI.
- Decision: Issuer configuration is stored in non-secret metadata storage and can be changed via UI settings.

## Risks / Trade-offs
- ACME library choice may affect portability → Mitigation: keep ACME concerns isolated behind `Issuer`.
- Confusion between staging and prod → Mitigation: strong visual affordance (banner/badge) and explicit naming.

## Migration Plan
- When adding production endpoints, require explicit opt-in and possibly an additional confirmation step.

## Open Questions
- Do we want per-issuer “account email/contact” stored as metadata?


