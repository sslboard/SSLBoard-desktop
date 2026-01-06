## 1. Backend Automation (Rust)
- [x] 1.1 `start_managed_issuance` returns a request id + DNS instructions and places automated DNS records where possible.
- [x] 1.2 `complete_managed_issuance` performs DNS propagation polling in the Rust core.
- [x] 1.3 `complete_managed_issuance` finalizes the ACME order and stores the certificate on success.
- [x] 1.4 Manual DNS is handled by returning `manual` DNS instructions; the UI proceeds after user confirmation via `complete_managed_issuance`.
- [x] 1.5 No background task progress events are required for this iteration.

## 2. Frontend Integration (TypeScript)
- [x] 2.1 `useManagedIssuanceFlow.ts` does not implement standalone DNS polling; it calls `completeManagedIssuance` to let the backend handle propagation checks.
- [x] 2.2 Issue page removes "Check Propagation" / step-by-step polling controls for automated providers.
- [x] 2.3 UI displays DNS instructions for manual records and provides a "Continue" action to proceed.
- [x] 2.4 No global background task tracking is required for this iteration.
- [x] 2.5 No persistent issuance task indicator is required for this iteration.
- [x] 2.6 Errors from `completeManagedIssuance` are surfaced in the Issue page.
