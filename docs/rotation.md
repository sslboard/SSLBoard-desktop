# Key Rotation Guidelines

This project keeps secrets inside the Rust core with only reference IDs exposed to the UI. Rotate keys deliberately with clear reasons and a reversible plan.

## ACME account keys (staging vs production)
- Maintain **separate account keys per environment** (staging vs production). Never reuse a staging key for production.
- **Do not rotate ACME account keys.** These keys are long-lived and should remain unchanged to preserve the ability to revoke certificates issued under the account. Rotating an ACME account key would create a new account, losing access to revoke certificates issued with the previous account key.
- If an ACME account key is compromised or must be replaced:
  - Create a new issuer configuration with a new account key (this creates a new ACME account).
  - Note that certificates issued under the old account cannot be revoked from the new account.
  - Consider the security implications before abandoning an account key.
- Safety rules:
  - UI never receives raw key bytes; all operations use secret references.
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
