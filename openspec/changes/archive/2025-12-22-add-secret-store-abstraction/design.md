## Context
This change introduces the project’s most important security boundary in code: secrets are stored and accessed only within the trusted Rust core, using OS-provided secure storage where available. The UI remains untrusted and sees only stable references.

## Goals / Non-Goals
- Goals:
  - Store secrets using OS secret stores by default.
  - Ensure secrets never cross the IPC boundary to the UI.
  - Provide stable reference identifiers so other modules can look up secrets inside Rust.
- Non-Goals:
  - Full user-presence gating and Secure Enclave integration (can be added later).
  - Cloud syncing of secrets (explicitly forbidden).

## Decisions
- Decision: Tauri commands MUST NOT return secret bytes to the UI; UI receives only reference ids and non-sensitive metadata (labels, createdAt, kind).
- Decision: Secrets SHOULD be namespaced and typed (e.g., `SecretKind::DnsCredential`) to reduce misuse.
- Decision: The local inventory store MUST NOT contain secret values; it may store only secret reference ids.

## Risks / Trade-offs
- OS secret store behavior differs by platform → Mitigation: keep an adapter layer and normalize error handling.
- Reference id leakage in logs/telemetry → Mitigation: treat refs as sensitive-ish identifiers; avoid logging them at info level.

## Migration Plan
- Start with a single adapter per platform; refactor into per-provider backends only when needed.

## Open Questions
- Do we need a user-visible label per secret ref from day 1?
- Do we want “replace secret value but keep ref id” semantics?


