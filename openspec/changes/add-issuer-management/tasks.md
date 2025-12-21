## 1. Implementation
- [x] 1.1 Define issuer entity model (DTO + storage record) with issuer type and parameters.
- [x] 1.2 Add or extend `issuance.sqlite` schema and migrations for issuer entities.
- [x] 1.3 Implement issuer CRUD commands (create/update/disable/select/list) with validation for ACME email + ToS acceptance.
- [x] 1.4 Seed Let's Encrypt staging and production issuers with staging selected by default.
- [x] 1.5 Update UI to manage issuers (add/edit/disable/select) and surface required ACME fields.
- [x] 1.6 Add basic validation/error states for missing email or ToS acceptance.
- [x] 1.7 Update docs (`docs/db.md`, settings/issuer flow docs) to reflect issuer entity model.
