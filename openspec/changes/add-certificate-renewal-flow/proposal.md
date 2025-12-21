# Change: Add Certificate Renewal Flow

## Why

Users need to renew expiring certificates without manually re-entering all the details. Renewal should pre-fill known information (SANs, issuer match) and optionally allow reusing the existing private key for scenarios where key continuity matters.

## What Changes

- Add a "Renew" action in certificate inventory/detail views that navigates to the Issue page with pre-filled data
- Pre-populate domains/SANs from the existing certificate
- Attempt to match the original issuer (by label/type) and pre-select it
- Offer an option to reuse the existing managed private key (when `managed_key_ref` exists) vs. generating a new key
- Track renewal lineage by optionally linking the new certificate to the previous one

## Impact

- Affected specs: `certificate-renewal` (new capability), may reference `certificate-issuance` patterns
- Affected code:
  - `src/pages/Issue.tsx` - Accept pre-fill props (domains, issuer hint, key reuse option)
  - `src/components/certificates/CertificateDetail.tsx` - Add "Renew" button
  - `src/components/certificates/InventoryEntry.tsx` - Optional quick "Renew" action
  - `src-tauri/src/issuance/flow.rs` - Support key reuse mode for managed issuance
  - `src-tauri/src/storage/inventory.rs` - Optional renewal lineage field

