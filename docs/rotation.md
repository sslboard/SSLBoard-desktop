# Key Rotation Guidelines

This project keeps secrets inside the Rust core with only reference IDs exposed to the UI. Rotate keys deliberately with clear reasons and a reversible plan.

## ACME account keys (staging vs production)
- Maintain **separate account keys per environment** (staging vs production). Never reuse a staging key for production.
- Default stance: **long-lived keys**, rotated only for: suspected compromise, ownership/organization changes, or when transferring accounts to a new custodian.
- Rotation flow:
  1) Generate a new account key via SecretStore (or import an existing ref). Keep the old ref until verification succeeds.
  2) Update issuer settings to point to the new ref and re-run `ensure_account` to register/verify.
  3) After confirmation, delete the old secret ref to avoid orphaned credentials.
- Safety rules:
  - UI never receives raw key bytes; all operations use secret references.
  - Staging rotation is safe to practice; production rotation should require explicit opt-in in UI copy.
  - Record the ref ID and timestamp for auditability (e.g., in metadata tables or logs when added).

## Managed private keys (issued certificates)
- Default to **reusing existing managed keys on renewal** unless policy or compromise requires rotation.
- When rotation is required, generate a new key locally and store it as a `managed_private_key` ref; keep the prior ref until the new certificate is verified and distributed.
- Avoid exporting private keys unless the user explicitly opts in (and warn when they do).

## DNS/API credentials (for completeness)
- Rotate when: provider suggests key rollover, access scope changes, or credentials are shared with other systems.
- Preserve least-privilege scopes and audit the mapping of credential ref → zones/records after rotation.

## Operational guardrails
- Require contact email and Terms of Service acceptance before touching production issuers.
- Keep a short-lived checklist in the UI for rotation steps to avoid silent footguns.
- Ensure deletions of old refs are intentional; best-effort cleanup happens only after successful replacement.
