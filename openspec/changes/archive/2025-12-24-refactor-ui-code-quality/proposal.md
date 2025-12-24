# Change: Refactor UI Code Quality

## Why
The UI code in `src/` has grown a few oversized, multi-responsibility components and hooks (e.g., issuance flow, issuer management, export modal). This increases cognitive load, raises effective cyclomatic complexity, and makes future feature work riskier. This change proposes a focused refactor to improve maintainability without changing product behavior or expanding the trust boundary.

## What Changes
- Split oversized UI modules into smaller, single-responsibility components and hooks (with clear naming and predictable boundaries).
- Extract repeated logic (validation, formatting, derived-state helpers) into shared modules under `src/lib/` and/or `src/hooks/`.
- Standardize UI conventions:
  - Prop handler naming (`onX` for props, `handleX` for local handlers).
  - Import grouping (external → internal, separated by blank lines).
  - Consistent error handling for user-facing messages.
- Remove dead code (unused exports, unused branches) surfaced by TypeScript strict checks.

## Impact
- **Affected specs**: `ui-code-quality` (new capability)
- **Affected code (planned)**:
  - `src/pages/Issue.tsx`
  - `src/components/settings/IssuerManager.tsx`
  - `src/components/certificates/CertificateExportModal.tsx`
  - `src/components/dns-providers/DnsProviderForm.tsx`
  - `src/hooks/useDnsProviderManager.ts`
  - new focused components/hooks under `src/components/**` and `src/hooks/**`
- **Breaking changes**: None (refactor only; UI behavior and IPC contracts remain stable)
- **Security posture**: No change. UI remains untrusted; secrets MUST stay in Rust core.


