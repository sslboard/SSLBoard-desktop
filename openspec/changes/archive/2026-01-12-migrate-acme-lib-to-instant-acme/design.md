## Context
The current issuance flow uses acme-lib. We want to migrate to instant-acme for a clearer ACME API surface and future extensibility while preserving existing behavior.

## Goals / Non-Goals
- Goals: migrate to instant-acme while preserving current issuance behavior and keeping secrets local.
- Non-Goals: add new user-facing ACME capabilities; change DNS provider integration.

## Decisions
- Decision: Replace acme-lib with instant-acme in the Rust core.
- Decision: Preserve existing issuer inputs, issuance flows, and error mapping where possible.

## Risks / Trade-offs
- Library migration could change error shapes or flow control; mitigate with focused tests and error mapping.

## Migration Plan
- Swap ACME client and validate issuance for staging providers.

## Open Questions
- None.
