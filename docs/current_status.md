# Current Status (Desktop)

## Recently Completed Changes (Archived)

- **UI Shell with Shadcn** (`2025-12-17-add-ui-shell-shadcn`): Complete UI shell implementation with routing, sidebar navigation, and Shadcn component library integration.
- **Secret Store Abstraction** (`2025-12-22-add-secret-store-abstraction`): OS keyring-backed secret storage with prefixed refs (`sec_`), metadata in SQLite, Tauri commands wired into Settings → Secrets UI.
- **Certificate Inventory Foundations** (`2025-12-22-add-certificate-inventory-foundations`): Inventory storage and UI with metadata-only listing, demo seed record support.
- **DNS Challenge Engine (Manual Adapter)** (`2025-12-22-add-dns-challenge-engine-manual-adapter`): DNS-01 manual adapter with propagation polling (ureq DoH lookup every 2s, 90s budget) and UI stepper on Issue page.
- **DNS Provider Adapters** (`2025-12-22-add-dns-provider-adapters`): Initial DNS provider adapter framework (currently stubbed implementations).
- **DNS Provider Configuration** (`2025-12-22-add-dns-provider-configuration`): End-to-end DNS provider management with new `dns_providers` storage, CRUD operations, test connection flow, and Settings page with overlap warnings.
- **Issue Certificate Flow** (`2025-12-22-add-issue-certificate-flow`): ACME issuance orchestration, managed-key path (RSA-2048), CSR generation, DNS-01 challenges, and Managed certificate persistence.
- **Issuer Management** (`2025-12-22-add-issuer-management`): ACME issuer store with Let's Encrypt staging/production, account key management, and UI settings integration.
- **ACME Issuer Sandbox** (`2025-12-22-add-acme-issuer-sandbox`): ACME account registration and order creation scaffolding.
- **Secrets Master Key Encryption Refactor** (`2025-12-22-refactor-secrets-master-key-encryption`): Master key encryption improvements.

## Active OpenSpec Changes

- **Certificate Renewal Flow** (`add-certificate-renewal-flow`): Not started - key reuse support, renewal lineage tracking, and UI integration for certificate renewals.
- **DNS Provider Integration Tests** (`add-dns-provider-integration-tests`): Partially complete - Cloudflare and DigitalOcean integration tests implemented, Route 53 tests and CI/documentation pending.
- **macOS Biometric Keychain** (`add-macos-biometric-keychain`): Not started - biometric access control for macOS using Touch ID/Face ID.
- **Rust Code Quality Refactoring** (`refactor-rust-code-quality`): **Fully Complete** - DNS provider code modularized, error handling improved, logging standardized, dead code removed, and performance/security enhancements implemented.
- **DNS Provider TXT Upsert Updates** (`update-dns-provider-txt-upsert`): Mostly complete - TXT content normalization and upsert behavior implemented for DigitalOcean/Route 53, validation pending.

## Current System State

- Secret storage: OS keyring-backed with metadata in `secrets.sqlite`; prefixed refs (`sec_`), secret kind `dns_provider_token` replaces `dns_credential`.
- Certificate inventory: Functional with metadata listing and demo seeding; real issuance flow implemented with ACME orchestration.
- DNS providers: Full CRUD management with test connection flow; provider resolution integrated into Issue flow with manual fallback.
- Issuance: End-to-end ACME flow with managed keys, DNS-01 challenges, and certificate persistence; staging/production issuer support.
- UI: Complete shell with routing, all major pages functional (Certificates, Issue, Settings with DNS Providers/Issuers/Secrets).
- Code quality: Recent refactoring completed - DNS modules split, error handling standardized, logging implemented, unused code cleaned up.

## Dependencies & Infrastructure

- Key dependencies: `uuid`, `keyring`, `tracing` (logging), `security_framework` (macOS biometric - pending).
- Databases: `secrets.sqlite` (secret metadata), `issuance.sqlite` (issuers), inventory in main DB.
- Build: `cargo fmt` + `cargo check` pass; no current compilation warnings.

## Pending/Placeholder Items

- Discover page: Still placeholder content.
- Certificate renewal: Not yet implemented (key reuse, lineage tracking, UI integration).
- DNS provider adapters: Currently stubbed - need real Cloudflare/DigitalOcean/Route 53 implementations.
- Integration tests: Route 53 tests and CI setup pending.
- macOS biometric features: Not yet implemented.
- ACME account key validation: May need additional error handling refinements.

## Next Logical Work

- Complete `add-dns-provider-integration-tests` (Route 53 tests, documentation, CI).
- Implement real DNS provider adapters (Cloudflare, DigitalOcean, Route 53).
- Add certificate renewal flow with key reuse and lineage tracking.
- Consider macOS biometric keychain enhancement.
- Flesh out ACME error handling and account key validation.
