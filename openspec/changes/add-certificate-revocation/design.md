## Context

ACME (RFC 8555) supports certificate revocation through the revocation endpoint. Revocation requires authentication using either:
1. The certificate's private key (keyCompromise, superseded, cessationOfOperation, or unspecified reasons)
2. The account key that originally issued the certificate (any reason)

Currently, certificates don't track which issuer was used, making revocation impossible. We need to:
- Track issuer association for Managed certificates
- Store revocation status to prevent re-issuance and display revocation state
- Implement revocation with proper authentication using available keys

## Goals / Non-Goals

**Goals:**
- Enable revocation of Managed certificates issued through the app
- Support revocation using either the certificate's private key or the issuer's account key
- Track revocation state in the certificate record
- Provide clear UI feedback on revocability status

**Non-Goals:**
- Revoking certificates issued outside the app (Discovered certificates)
- Revoking certificates where the issuer is unknown or no longer configured
- Automatic revocation workflows or scheduled revocation
- Revocation reasons beyond ACME standard (keyCompromise, superseded, cessationOfOperation, unspecified)

## Decisions

**Decision: Track issuer_id on certificate records**
- Store `issuer_id` as a nullable field (NULL for Discovered certificates)
- Populate during issuance with the issuer used
- Migration: existing certificates will have NULL issuer_id (not revocable)

**Decision: Use private key for revocation authentication when available**
- Prefer certificate's private key (`managed_key_ref`) for revocation
- Fall back to issuer's account key if private key unavailable but issuer account key exists
- Require at least one key to be available for revocation

**Decision: Store revocation metadata**
- Add `revoked_at: Option<DateTime<Utc>>` timestamp
- Add `revocation_reason: Option<String>` (ACME reason codes: "keyCompromise", "superseded", "cessationOfOperation", "unspecified")
- Once revoked, mark as non-revocable in UI

**Decision: ACME revocation reason handling**
- Default to "unspecified" if user doesn't provide a reason
- Support all ACME standard revocation reasons
- UI may offer dropdown or default to unspecified for simplicity (future enhancement)

**Alternatives considered:**
- Track revocation separately from certificate records: rejected, revocation is part of certificate lifecycle
- Require both keys: rejected, ACME only requires one authentication method
- Auto-revoke on deletion: rejected, revocation is explicit security action requiring intent

## Risks / Trade-offs

**Risk: Existing certificates can't be revoked**
- **Mitigation**: Document limitation, only new certificates after this change will support revocation

**Risk: Issuer deletion prevents revocation**
- **Mitigation**: Warn users before deleting issuers with issued certificates, or prevent deletion when active certificates exist

**Risk: Revocation is irreversible**
- **Mitigation**: Add confirmation dialog in UI, clear messaging about permanence

**Risk: Revocation may fail due to network/CA issues**
- **Mitigation**: Return clear error messages, allow retry, don't mark as revoked locally until confirmed by CA

## Migration Plan

1. Add `issuer_id`, `revoked_at`, `revocation_reason` columns to `certificate_records` table
2. Populate `issuer_id` for all new issuances going forward
3. Existing certificates with NULL `issuer_id` are not revocable (by design)
4. UI checks for revocability before showing Revoke button

## Open Questions

- Should we attempt to populate `issuer_id` retroactively for existing certificates based on heuristics (directory_url matching, etc.)? **Answer: No, too risky without certainty**
- Should revocation require a confirmation dialog with reason selection? **Answer: Yes, at minimum a confirmation; reason selection can be simplified initially**
