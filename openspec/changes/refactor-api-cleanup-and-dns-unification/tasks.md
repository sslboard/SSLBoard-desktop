## 1. Backend Changes (Rust)
- [ ] 1.1 Remove `seed_fake_certificate` and `greet` from `src-tauri/src/lib.rs` and command modules.
- [ ] 1.2 Remove `prepare_dns_challenge` and `check_dns_propagation` from exported commands.
- [ ] 1.3 Remove `lock_vault` and `is_vault_unlocked` from exported commands.
- [ ] 1.4 Remove `create_secret_ref`, `update_secret_ref`, and `delete_secret_ref` from exported commands.
- [ ] 1.5 Update `dns_provider_test` to include initial credential validation before proceeding to propagation test.
- [ ] 1.6 Remove `dns_provider_validate_token` command.

## 2. Frontend Changes (TypeScript)
- [ ] 2.1 Update `src/lib/` to remove deleted command wrappers.
- [ ] 2.2 Update `src/hooks/useVaultControls.ts` to remove reliance on explicit vault commands and state.
- [ ] 2.3 Remove Vault lock/unlock UI components from `sidebar.tsx` or `topbar.tsx`.
- [ ] 2.4 Update `src/hooks/useDnsProviderManager.ts` and `src/hooks/useDnsProviderTokenTest.ts` to use unified `testDnsProvider`.
- [ ] 2.5 Update DNS provider forms to remove separate "Validate" buttons.
- [ ] 2.6 Update `src/hooks/useManagedIssuanceFlow.ts` to remove manual DNS propagation checking logic.
- [ ] 2.7 Clean up UI pages and components referring to removed commands.

