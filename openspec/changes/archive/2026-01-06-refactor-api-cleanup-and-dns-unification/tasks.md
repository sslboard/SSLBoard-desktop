## 1. Backend Changes (Rust)
- [x] 1.1 Remove `seed_fake_certificate` and `greet` from `src-tauri/src/lib.rs` and command modules.
- [x] 1.2 Remove `prepare_dns_challenge` and `check_dns_propagation` from exported commands (remain internal to issuance workflow).
- [x] 1.3 Remove `is_vault_unlocked` from exported commands (vault state is event-driven in the UI).
- [x] 1.4 Remove `create_secret_ref`, `update_secret_ref`, and `delete_secret_ref` from exported commands.
- [x] 1.5 Use a single `dns_provider_test` command that performs an end-to-end TXT write + propagation check; auth failures are surfaced as part of the same test result.
- [x] 1.6 Remove `dns_provider_validate_token` command.

## 2. Frontend Changes (TypeScript)
- [x] 2.1 Update `src/lib/` to remove deleted command wrappers.
- [x] 2.2 Keep vault UX lightweight: show status indicator and support auto-lock; avoid vault-gating overlays or mandatory manual unlock steps.
- [x] 2.3 Remove Vault lock/unlock UI buttons from primary navigation (status-only indicator is permitted).
- [x] 2.4 Use unified `testDnsProvider` flow for DNS provider testing.
- [x] 2.5 DNS provider forms use a single "Test Connection" action.
- [x] 2.6 Managed issuance flow does not require manual DNS propagation polling via standalone commands.
- [x] 2.7 Clean up UI pages and components referring to removed commands.
