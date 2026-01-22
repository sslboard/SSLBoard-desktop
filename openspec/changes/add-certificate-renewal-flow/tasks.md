## 1. Backend: Key Reuse Support

- [x] 1.1 Extend `StartIssuanceRequest` to accept optional `reuse_key_ref: Option<String>` for key reuse mode
- [x] 1.2 Modify `start_managed_issuance` to use the referenced key (if provided and valid) instead of generating a new one
- [x] 1.3 Add validation that the referenced key exists in SecretStore before proceeding

## 2. Backend: Renewal Lineage Tracking (Optional)

- [x] 2.1 Add `renewed_from: Option<String>` field to `CertificateRecord` schema
- [x] 2.2 Migrate existing inventory table to include the new column
- [x] 2.3 Populate `renewed_from` when completing a renewal-originated issuance

## 3. Frontend: Issue Page Pre-fill Support

- [x] 3.1 Add route state/query params to Issue page for pre-fill data (domains, issuer_hint, key_ref, renewing_cert_id)
- [x] 3.2 Parse and apply pre-fill values to form state on mount
- [x] 3.3 Add issuer matching logic to auto-select issuer by label/type when `issuer_hint` is provided
- [x] 3.4 Add UI toggle for "Reuse existing private key" when `key_ref` is available

## 4. Frontend: Certificate Detail/Inventory Renew Actions

- [x] 4.1 Add "Renew" button to CertificateDetail component
- [x] 4.2 Wire button to navigate to Issue page with pre-fill state (SANs, issuer label, managed_key_ref if present)
- [x] 4.3 Optionally add quick "Renew" action in InventoryEntry row for Managed certificates nearing expiration

## 5. UX Polish

- [x] 5.1 Show visual indicator when renewing (e.g., "Renewing: example.com" in Issue page header)
- [x] 5.2 Display key reuse option only for Managed certificates with a managed_key_ref
- [x] 5.3 Add tooltip explaining tradeoffs of key reuse vs. new key generation
