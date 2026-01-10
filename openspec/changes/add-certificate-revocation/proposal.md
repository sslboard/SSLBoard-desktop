# Change: Add Certificate Revocation

## Why

Users need to revoke certificates that were previously issued, either due to compromise, changes in requirements, or other operational needs. Revocation is a critical security operation that must be performed at the issuer level using proper authentication (private key or issuer account key).

## What Changes

- Add `issuer_id` field to certificate records to track which issuer was used for issuance (required for revocation)
- Add revocation tracking fields to certificate records (`revoked_at`, `revocation_reason`)
- Implement revocation command in Rust core that communicates with the ACME issuer's revocation endpoint
- Add "Revoke" button in certificate detail UI (only for revocable certificates)
- Track revocation state and prevent re-issuance attempts on revoked certificates
- Support revocation only for Managed certificates with valid issuer configuration and required keys

## Impact

- Affected specs: `certificate-issuance` (MODIFIED), new revocation capability
- Affected code:
  - `src-tauri/src/core/types.rs` - Add `issuer_id`, `revoked_at`, `revocation_reason` to `CertificateRecord`
  - `src-tauri/src/storage/migrations.rs` - Add migration for new certificate record columns
  - `src-tauri/src/storage/inventory.rs` - Update schema and queries
  - `src-tauri/src/issuance/flow.rs` - Store `issuer_id` during issuance
  - `src-tauri/src/core/commands/` - New revocation command
  - `src-tauri/src/issuance/` - ACME revocation implementation
  - `src/components/certificates/CertificateDetail.tsx` - Add "Revoke" button
  - `src/lib/certificates.ts` - Add revocation API call
