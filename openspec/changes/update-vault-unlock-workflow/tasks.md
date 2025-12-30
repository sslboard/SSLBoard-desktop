## 1. Backend Changes

- [ ] 1.1 Review and document that `ensure_unlocked()` already handles auto-unlock
- [ ] 1.2 Ensure error handling for locked vault scenarios is clear and user-friendly
- [ ] 1.3 Verify `unlock_vault` command behavior (may keep for internal use or remove)

## 2. Frontend Changes

- [ ] 2.1 Update `Topbar` component to remove unlock button, keep lock button and status indicator
- [ ] 2.2 Update `useVaultControls` hook to remove `toggleVault`, add separate `lockVault` function
- [ ] 2.3 Remove `unlockVault` from public API in `src/lib/secrets.ts` (or mark as deprecated)
- [ ] 2.4 Update UI to show locked state without requiring unlock action
- [ ] 2.5 Ensure error messages for locked vault operations are clear

## 3. Error Handling & UX

- [ ] 3.1 Test error handling when vault is locked and operation is attempted
- [ ] 3.2 Verify error messages guide users appropriately
- [ ] 3.3 Ensure vault state updates are reflected in UI after auto-unlock
- [ ] 3.4 Test idle timeout lock behavior still works correctly

## 4. Testing

- [ ] 4.1 Test operations automatically unlock vault when needed
- [ ] 4.2 Test lock button still works for explicit locking
- [ ] 4.3 Test idle timeout auto-lock behavior
- [ ] 4.4 Test error scenarios (keyring unavailable, auth failures)
- [ ] 4.5 Verify UI state updates correctly after auto-unlock

## 5. Documentation

- [ ] 5.1 Update user-facing documentation about vault behavior
- [ ] 5.2 Document that unlock happens automatically when needed
- [ ] 5.3 Update developer docs about vault unlock workflow

