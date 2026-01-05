## 1. Backend Changes

- [x] 1.1 Review and document that `ensure_unlocked()` already handles auto-unlock
- [x] 1.2 Ensure error handling for locked vault scenarios is clear and user-friendly
- [x] 1.3 Verify `unlock_vault` command behavior (may keep for internal use or remove)

## 2. Frontend Changes

- [x] 2.1 Update `Topbar` component to remove unlock button, keep lock button and status indicator
- [x] 2.2 Update `useVaultControls` hook to remove `toggleVault`, add separate `lockVault` function
- [x] 2.3 Remove `unlockVault` from public API in `src/lib/secrets.ts` (or mark as deprecated)
- [x] 2.4 Update UI to show locked state without requiring unlock action
- [x] 2.5 Ensure error messages for locked vault operations are clear

## 3. Error Handling & UX

- [x] 3.1 Test error handling when vault is locked and operation is attempted
- [x] 3.2 Verify error messages guide users appropriately
- [x] 3.3 Ensure vault state updates are reflected in UI after auto-unlock
- [x] 3.4 Test idle timeout lock behavior still works correctly

## 4. Testing

- [x] 4.1 Test operations automatically unlock vault when needed
- [x] 4.2 Test lock button still works for explicit locking
- [x] 4.3 Test idle timeout auto-lock behavior
- [x] 4.4 Test error scenarios (keyring unavailable, auth failures)
- [x] 4.5 Verify UI state updates correctly after auto-unlock

## 5. Documentation

- [x] 5.1 Update user-facing documentation about vault behavior
- [x] 5.2 Document that unlock happens automatically when needed
- [x] 5.3 Update developer docs about vault unlock workflow
