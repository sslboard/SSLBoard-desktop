## 1. Backend: Track Issuer on Certificate Records

- [ ] 1.1 Add `issuer_id: Option<String>` field to `CertificateRecord` in `src-tauri/src/core/types.rs`
- [ ] 1.2 Add `revoked_at: Option<DateTime<Utc>>` field to `CertificateRecord`
- [ ] 1.3 Add `revocation_reason: Option<String>` field to `CertificateRecord`
- [ ] 1.4 Add database migration for new columns in `src-tauri/src/storage/migrations.rs`
- [ ] 1.5 Update `InventoryStore` schema and insert/update methods to handle new fields
- [ ] 1.6 Modify `build_record` in `src-tauri/src/issuance/flow.rs` to accept and store `issuer_id`

## 2. Backend: Pass Issuer ID During Issuance

- [ ] 2.1 Update `start_managed_issuance` command to accept and store `issuer_id` with certificate record
- [ ] 2.2 Ensure `issuer_id` is passed from issuance flow to record building
- [ ] 2.3 Update issuance command signatures/types as needed

## 3. Backend: Implement Revocation Command

- [ ] 3.1 Create `RevokeCertificateRequest` DTO with `certificate_id` and optional `revocation_reason`
- [ ] 3.2 Create `revoke_certificate` Tauri command in `src-tauri/src/core/commands/`
- [ ] 3.3 Implement revocation logic that:
  - Validates certificate exists and is Managed source
  - Validates `issuer_id` is present and issuer exists
  - Checks for required keys (managed_key_ref or issuer account_key_ref)
  - Loads issuer configuration and account key
  - Constructs ACME revocation request using acme-lib
  - Submits revocation to CA
  - Updates certificate record with revocation metadata on success
- [ ] 3.4 Handle revocation errors gracefully (network, CA rejection, etc.)
- [ ] 3.5 Add revocation reason constants/enum for ACME standard reasons

## 4. Backend: ACME Revocation Implementation

- [ ] 4.1 Research acme-lib revocation API (check Certificate type methods)
- [ ] 4.2 Implement revocation using certificate private key method
- [ ] 4.3 Implement revocation using account key method (fallback)
- [ ] 4.4 Add revocation endpoint URL construction from issuer directory URL
- [ ] 4.5 Test revocation against staging ACME server

## 5. Frontend: Certificate Type Updates

- [ ] 5.1 Add `issuer_id`, `revoked_at`, `revocation_reason` to `CertificateRecord` TypeScript type in `src/lib/certificates.ts`
- [ ] 5.2 Update certificate fetching/display logic to handle new fields
- [ ] 5.3 Add helper function to determine if certificate is revocable

## 6. Frontend: Revocation API Integration

- [ ] 6.1 Add `revokeCertificate` function in `src/lib/certificates.ts`
- [ ] 6.2 Create IPC call to revocation command
- [ ] 6.3 Handle revocation success/error responses

## 7. Frontend: UI Revocation Button

- [ ] 7.1 Add "Revoke" button to `CertificateDetail` component alongside "Export" button
- [ ] 7.2 Show button only when certificate is revocable (Managed source, has issuer_id, not already revoked, required keys available)
- [ ] 7.3 Add confirmation dialog before revocation
- [ ] 7.4 Show revocation status in certificate details when revoked
- [ ] 7.5 Display revocation date and reason if available
- [ ] 7.6 Disable Revoke button after successful revocation

## 8. UX Polish

- [ ] 8.1 Add visual indicator for revoked certificates (badge/status)
- [ ] 8.2 Show appropriate error messages when revocation is not possible (missing issuer, missing keys, already revoked)
- [ ] 8.3 Ensure revocation action is clearly labeled and destructive (red button or warning styling)
