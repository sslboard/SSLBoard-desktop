## 1. Backend: Track Issuer on Certificate Records

- [x] 1.1 Add `issuer_id: Option<String>` field to `CertificateRecord` in `src-tauri/src/core/types.rs`
- [x] 1.2 Add `revoked_at: Option<DateTime<Utc>>` field to `CertificateRecord`
- [x] 1.3 Add `revocation_reason: Option<String>` field to `CertificateRecord`
- [x] 1.4 Add database migration for new columns in `src-tauri/src/storage/migrations.rs`
- [x] 1.5 Update `InventoryStore` schema and insert/update methods to handle new fields
- [x] 1.6 Modify `build_record` in `src-tauri/src/issuance/flow.rs` to accept and store `issuer_id`

## 2. Backend: Pass Issuer ID During Issuance

- [x] 2.1 Update `start_managed_issuance` command to accept and store `issuer_id` with certificate record
- [x] 2.2 Ensure `issuer_id` is passed from issuance flow to record building

## 3. Backend: Implement Revocation Command

- [x] 3.1 Create `RevokeCertificateRequest` DTO with `certificate_id` and optional `revocation_reason`
- [x] 3.2 Create `revoke_certificate` Tauri command in `src-tauri/src/core/commands/`
- [x] 3.3 Implement revocation logic that:
  - Validates certificate exists and is Managed source
  - Validates `issuer_id` is present and issuer exists
  - Checks for required keys (managed_key_ref or issuer account_key_ref)
  - Loads issuer configuration and account key
  - Constructs ACME revocation request using instant-acme
  - Submits revocation to CA
  - Updates certificate record with revocation metadata on success
- [x] 3.4 Handle revocation errors gracefully (network, CA rejection, etc.)
- [x] 3.5 Add revocation reason constants/enum for ACME standard reasons
- [x] 3.6 Validate revocation reason allowlist server-side and default to "unspecified" when missing

## 4. Backend: ACME Revocation Implementation

- [x] 4.1 Research instant-acme revocation API (check revocation request options)
- [x] 4.3 Implement revocation using account key method (fallback)

## 5. Frontend: Certificate Type Updates

- [x] 5.1 Add `issuer_id`, `revoked_at`, `revocation_reason` to `CertificateRecord` TypeScript type in `src/lib/certificates.ts`
- [x] 5.2 Update certificate fetching/display logic to handle new fields

## 6. Frontend: Revocation API Integration

- [x] 6.1 Add `revokeCertificate` function in `src/lib/certificates.ts`
- [x] 6.2 Create IPC call to revocation command
- [x] 6.3 Handle revocation success/error responses

## 7. Frontend: UI Revocation Button

- [x] 7.1 Add "Revoke" button to `CertificateDetail` component alongside "Export" button
- [x] 7.2 Show button only when certificate is revocable (Managed source, has issuer_id, not already revoked, required keys available)
- [x] 7.3 Add confirmation dialog before revocation
- [x] 7.4 Add revocation reason dropdown (ACME reasons only, no free text) in confirmation dialog
- [x] 7.5 Show revocation status in certificate details when revoked
- [x] 7.6 Display revocation date and reason if available
- [x] 7.7 Disable Revoke button after successful revocation

## 8. UX Polish

- [x] 8.1 Add visual indicator for revoked certificates (badge/status)
- [x] 8.2 Show appropriate error messages when revocation is not possible (missing issuer, missing keys, already revoked)
- [x] 8.3 Ensure revocation action is clearly labeled and destructive (red button or warning styling)
- [x] 8.4 Add warning dialog before deleting issuers that have issued certificates
