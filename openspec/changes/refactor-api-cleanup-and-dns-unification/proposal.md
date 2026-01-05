# Change: Refactor API Cleanup and DNS Unification

## Why

The current API surface contains several redundant or development-only commands that complicate the contract between the frontend and backend. Consolidating DNS testing/validation and removing unnecessary commands like seeding and manual challenge preparation will simplify UI development and align with a more automated backend-driven workflow.

## What Changes

- **BREAKING**: REMOVE `seed_fake_certificate` and `greet` commands.
- **BREAKING**: REMOVE `prepare_dns_challenge` and `check_dns_propagation` as standalone UI-callable commands (functionality moves to backend issuance workflow).
- **BREAKING**: REMOVE explicit `is_vault_unlocked` and `lock_vault` from the required UI state management (vault management becomes transparent).
- **BREAKING**: REMOVE `create_secret_ref` and `delete_secret_ref` from primary UI flows (secrets are managed implicitly by provider/issuer commands).
- **BREAKING**: CONSOLIDATE `dns_provider_test` and `dns_provider_validate_token` into a unified `dns_provider_test` command that performs both credential validation and propagation testing.
- **UI**: REMOVE manual vault lock/unlock controls and status indicators from the app shell.
- **UI**: SIMPLIFY DNS provider forms to use a single "Test Connection" action.

## Impact

- Affected specs: `certificate-issuance`, `dns-configuration`, `ui-shell`, `rust-code-quality`
- Affected code: `src-tauri/src/core/commands/`, `src/lib/`, `src/hooks/`

