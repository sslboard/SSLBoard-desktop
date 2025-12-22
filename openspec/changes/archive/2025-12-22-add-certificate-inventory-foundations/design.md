## Context

This change establishes the first durable local state: a certificate inventory of **metadata only**. It must uphold the project trust boundary: UI is untrusted; secrets and sensitive keys stay in Rust.

## Goals / Non-Goals

- Goals:
  - Persist and query certificate inventory records across restarts.
  - Provide stable read APIs for UI to render list + details.
  - Keep the store strictly non-secret (no private keys, no raw credentials).
- Non-Goals:
  - Issuance, renewal, export, or CT discovery (covered by later steps).
  - Full schema migration engine (acceptable to start simple; add migrations later).

## Decisions

- Decision: The initial storage backend SHOULD be SQLite to support filtering/sorting and future history/audit tables.
- Decision: UI MUST NOT access the storage directly; all access is via Tauri commands returning DTOs.

## Risks / Trade-offs

- Schema evolution risk → Mitigation: start with a small schema; keep versioned migrations once additional tables appear.
- Over-modeling early → Mitigation: store only inventory metadata required by UI, not every PKI field.

## Migration Plan

- If starting with a stub (JSON or in-memory), migrate to SQLite by adding a one-time import on startup.

## Open Questions

- Do we want inventory “versions” per certificate lineage, or a flat list for v0?


