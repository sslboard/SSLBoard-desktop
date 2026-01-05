# Change: Update Vault Unlock Workflow to Backend-Driven

## Why

The current vault unlock workflow requires users to explicitly unlock the vault via a UI button before performing operations that need secrets. With biometric authentication (Touch ID/Face ID), this creates a suboptimal user experience where users see prompts twice: once when clicking unlock, and again when accessing secrets. Moving unlock to be backend-driven (on-demand) provides a more natural macOS-like experience where authentication happens automatically when secrets are actually needed.

## What Changes

- Remove explicit unlock button from frontend UI
- Backend automatically unlocks vault on-demand when operations require secrets
- Vault unlock triggers authentication (biometric or keyring) transparently
- Frontend displays vault lock status but doesn't require manual unlock action
- Operations that need secrets automatically trigger unlock with appropriate authentication prompts
- Lock button remains for explicit user-initiated locking

## Impact

- Affected specs: `secret-store` (modify vault unlock workflow requirements)
- Affected code:
  - `src/components/layout/topbar.tsx` - Remove unlock button, keep lock button and status display
  - `src/hooks/useVaultControls.ts` - Remove unlock toggle, keep lock functionality and status tracking
  - `src/lib/secrets.ts` - Remove `unlockVault` export (keep for internal use if needed)
  - `src-tauri/src/core/commands/secrets.rs` - Keep `unlock_vault` command but document it's for internal use
  - `src-tauri/src/secrets/manager.rs` - `ensure_unlocked()` already handles auto-unlock (no changes needed)

This change improves UX by eliminating the need for explicit unlock actions and aligns with macOS security patterns where authentication happens on-demand.

