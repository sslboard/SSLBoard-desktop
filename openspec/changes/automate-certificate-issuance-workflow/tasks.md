## 1. Backend Automation (Rust)
- [ ] 1.1 Refactor `start_managed_issuance` to spawn an asynchronous task for the full issuance lifecycle.
- [ ] 1.2 Implement backend DNS propagation polling logic within the issuance task.
- [ ] 1.3 Implement automatic ACME order finalization upon propagation success.
- [ ] 1.4 Add support for pausing the task and emitting events when manual DNS configuration is required.
- [ ] 1.5 Implement progress events (e.g., `issuance-progress`) to update the UI on current state (DNS_PENDING, PROPAGATING, FINALIZING, COMPLETED, FAILED).

## 2. Frontend Integration (TypeScript)
- [ ] 2.1 Update `useManagedIssuanceFlow.ts` to listen for backend progress events instead of manual polling.
- [ ] 2.2 Simplify the Issue page UI to reflect the automated background process (remove step-by-step manual controls).
- [ ] 2.3 Implement UI handlers for manual intervention requests from the backend (display DNS instructions and "Confirm" button).
- [ ] 2.4 Add a background task provider or global hook to track issuance status across the app.
- [ ] 2.5 Add a persistent status indicator to the `sidebar.tsx` or `topbar.tsx` for active issuance tasks.
- [ ] 2.6 Add error handling for background task failures reported via events.

