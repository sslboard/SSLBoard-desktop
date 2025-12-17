## 1. Implementation
- [x] 1.1 Define `issuance::issuer::Issuer` interface and shared types (order id, authorization/challenge DTOs, finalize inputs).
- [x] 1.2 Implement ACME issuer using Let’s Encrypt **staging** endpoint as the default configuration.
- [x] 1.3 Implement `ensure_account()` that creates (or loads) an ACME account and stores the account key in `SecretStore`, persisting only a secret reference.
- [x] 1.4 Persist `issuer_id` and issuer configuration in the local metadata store.
- [x] 1.5 Add UI “Issuer” settings with a dropdown and a prominent “Sandbox” banner/badge when staging is selected.

