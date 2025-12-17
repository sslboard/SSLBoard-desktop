## 1. Implementation
- [ ] 1.1 Add issuance orchestration in Rust to start ACME orders, compute DNS-01 challenges, and drive finalize/download after propagation success.
- [ ] 1.2 Implement CSR validation and parsing (SAN/domain extraction) and surface clear errors for unsupported or mismatched names.
- [ ] 1.3 Add managed-key path: generate private key, store in `SecretStore`, create CSR internally, and keep key material out of the UI.
- [ ] 1.4 Persist issued certificate chain + metadata in inventory as `Managed`, recording key refs when present and noting CSR-import cases.
- [ ] 1.5 Build UI issuance wizard: domains entry, key-mode selection (generate vs import CSR), DNS-01 stepper with manual adapter, finalize and success screen.
