## 1. Implementation
- [x] 1.1 Add a non-secret metadata store module under `src-tauri/src/storage/` (SQLite preferred; allow a minimal stub for early dev).
- [x] 1.2 Define `CertificateRecord` in Rust (DTO + persisted shape) including: id, subjects/SANs, issuer, serial, not_before/not_after, fingerprint, source (`External|Managed`), domain roots, tags.
- [x] 1.3 Implement `list_certificates` and `get_certificate(id)` in Rust core and expose as typed Tauri commands.
- [x] 1.4 Add initial UI “Certificates” screen: table + details panel + empty-state CTA(s).
- [x] 1.5 Add minimal seed/dev helper to insert a fake certificate record for manual testing.

