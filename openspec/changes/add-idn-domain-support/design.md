## Context
Issuance currently rejects IDNs because domain validation treats non-ASCII characters as invalid. ACME and DNS APIs require ASCII labels (punycode/A-labels), but the UI should remain user-friendly with Unicode input and display.

## Goals / Non-Goals
- Goals: Accept Unicode domains in the UI, convert to punycode in the Rust core for ACME/DNS, and keep display names human-friendly.
- Non-Goals: Adding UI localization or expanded text-direction handling beyond existing UI patterns.

## Decisions
- Decision: Normalize domain labels using IDNA processing in the Rust core before any ACME or DNS operations.
- Decision: Store ASCII (punycode) forms for internal matching and persistence, while retaining Unicode for UI display.
- Decision: Update DNS zone suffix matching logic to compare IDNA-normalized forms.

## Risks / Trade-offs
- Risk: Unicode look-alike characters can obscure visual verification. Mitigation: continue to show the exact user-entered Unicode string without normalization in the UI, and keep validation logic in the Rust core.

## Migration Plan
No data migration required. New requests will normalize IDNs on ingest.

## Open Questions
- None.
