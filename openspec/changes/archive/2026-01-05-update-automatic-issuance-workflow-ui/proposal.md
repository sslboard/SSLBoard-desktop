# Change: Automatic issuance workflow UI

## Why
The current issuance flow requires three separate button clicks, which creates unnecessary friction and makes it harder to understand progress. A single explicit start with automatic step progression improves clarity while keeping the user in control.

## What Changes
- Replace the multi-button issuance flow (Start Issuance, Check DNS, Finish) with a single Start action that runs the full workflow.
- Display step-by-step progress that shows each stage running and its result, including manual DNS instructions when required.
- Surface a completed certificate summary on success or a clear error state with retry options on failure.

## Impact
- Affected specs: certificate-issuance
- Affected code: Issue page workflow UI, issuance progress state handling
