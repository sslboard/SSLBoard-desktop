## Context

Managed issuance currently generates RSA-2048 keys only. Users need stronger RSA sizes and ECC curves while keeping the UI untrusted and the Rust core in control of key generation.

## Goals / Non-Goals

- Goals:
  - Support RSA 2048/3072/4096 and ECDSA P-256/P-384 for managed issuance.
  - Keep key selection validated in Rust before any issuance work.
  - Preserve backwards compatibility for callers that do not send key parameters.
- Non-Goals:
  - Support custom key sizes/curves beyond the allowed list.
  - Add CSR-import key options or endpoint-generated key flows in this change.

## Decisions

- Decision: Extend the managed issuance request with explicit key parameters.
  - `key_algorithm`: `rsa` or `ecdsa`.
  - `key_size`: required for RSA (2048, 3072, 4096).
  - `key_curve`: required for ECDSA (p256, p384).
- Decision: Default to RSA-2048 when key parameters are omitted.
- Decision: Validate the parameter combination in Rust and return a clear error if invalid.
- Decision: Persist selected key algorithm parameters in certificate metadata for UI display and filtering.

## Risks / Trade-offs

- Risk: UI is untrusted and could submit unsupported values.
  - Mitigation: Strict Rust validation and enum-based DTOs where possible.
- Trade-off: Limited list of options to minimize complexity and policy risk.

## Migration Plan

- Backward compatibility: if the request lacks key parameters, treat it as RSA-2048.
- UI adds selection controls but defaults to the current behavior.

## Open Questions

- None.
