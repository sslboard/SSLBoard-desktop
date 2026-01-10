## 1. Implementation
- [ ] 1.1 Define CSR DTOs and IPC commands for CSR import, CSR generation, and CSR-based issuance.
- [ ] 1.2 Implement CSR parsing and validation in the Rust core (signature, SANs, key type) and surface validation errors.
- [ ] 1.3 Implement CSR-based issuance path that uses CSR-derived SANs and key metadata.
- [ ] 1.4 Implement CSR generation with managed key creation, CSR PEM output, and file write to user-selected path.
- [ ] 1.5 Update Issue UI to support CSR import and CSR creation workflows.
- [ ] 1.6 Persist CSR metadata on certificate records and surface it in the completion view.
- [ ] 1.7 Add focused Rust tests for CSR parsing, validation, and generation.
