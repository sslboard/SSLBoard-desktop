## 1. Implementation
- [x] 1.1 Add issuance orchestration in Rust to start ACME orders, compute DNS-01 challenges, and drive finalize/download after propagation success.
- [x] 1.2 Implement managed-key path: generate private key (RSA-2048 default), store in `SecretStore`, create CSR internally, and keep key material out of the UI.
- [x] 1.3 Validate domain/SAN inputs and surface clear errors for empty or unsupported names before creating orders.
- [x] 1.4 Persist issued certificate chain + metadata in inventory as `Managed`, recording key refs.
- [x] 1.5 Build UI issuance wizard: domains entry, managed-key path, DNS-01 stepper with manual adapter, finalize and success screen.
