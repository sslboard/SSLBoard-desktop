## Context
Issuer configuration today is effectively a fixed list in settings. The project already stores ACME-related issuer data in `issuance.sqlite`, but adding new issuers with distinct parameters needs a first-class entity model.

## Goals / Non-Goals
- Goals:
  - Allow users to add issuer entries with issuer-specific parameters.
  - Keep Let's Encrypt staging as the safe default; production requires explicit opt-in.
  - Preserve the trust boundary: issuer secrets remain in `SecretStore` references only.
- Non-Goals:
  - Implement new issuer types beyond Let's Encrypt ACME in this change.
  - Change issuance flows or ACME order handling.

## Decisions
- Decision: Keep the existing `issuer_configs` table name and extend it as needed.
- Decision: Model issuers as first-class records with common fields (id, label, type, environment, state, timestamps) plus issuer-specific parameters stored in a flexible SQL payload, while mapping to typed structs in Rust.
- Decision: Require contact email and explicit ToS acceptance at issuer creation time; ToS acceptance is sufficient for production (no extra confirmation).
- Decision: Issuer entries are editable after creation (label/environment/params) with validation.

## Risks / Trade-offs
- Adding flexible issuer parameters introduces schema decisions (typed columns vs JSON payloads).
- Issuer creation UI adds user-facing complexity; validation must remain clear and safe.

## Migration Plan
- If the issuer table already exists, add any missing columns via migrations (e.g., `issuer_type`, `params_json`, or `tos_agreed`).
- Seed Let's Encrypt staging/prod records if missing; ensure staging is selected when no enabled issuer exists.

## Open Questions
- None.
