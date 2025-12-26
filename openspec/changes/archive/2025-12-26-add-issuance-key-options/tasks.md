## 1. Implementation
- [x] 1.1 Add key algorithm fields to managed issuance DTOs in Rust and TypeScript.
- [x] 1.2 Implement Rust validation for allowed RSA sizes (2048/3072/4096) and ECDSA curves (P-256/P-384).
- [x] 1.3 Use requested key parameters during managed key generation in issuance flow.
- [x] 1.4 Update Issue page UI to expose key type/size selection with sensible defaults.
- [x] 1.5 Ensure Issue page passes key options into `start_managed_issuance` requests.
- [x] 1.6 Persist selected key algorithm parameters in certificate metadata for display/filtering.
- [x] 1.7 Add or update tests for key option validation (Rust) and UI request shaping (as feasible).
