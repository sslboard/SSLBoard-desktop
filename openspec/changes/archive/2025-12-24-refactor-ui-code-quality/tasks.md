# Implementation Tasks for UI Code Quality Refactor

## 1. Issue page refactor (`src/pages/Issue.tsx`)
- [x] 1.1 Create orchestration hook `src/hooks/useManagedIssuanceFlow.ts`
  - Own the step state: domains input → start issuance → check propagation → finalize → success
  - Expose a minimal view-model API (data + handlers) consumed by the page
  - Preserve current behavior and error messages
- [x] 1.2 Split UI sections into components under `src/components/issue/`
  - `IssuerSelectionCard.tsx` (issuer picker + readiness banner)
  - `DomainsInputCard.tsx` (domains textarea + start/reset actions)
  - `DnsProviderPreviewCard.tsx` (provider preview render)
  - `DnsInstructionsPanel.tsx` (manual/managed instructions + check/finalize controls)
  - `IssuanceResultBanner.tsx` (success/error banners)
- [x] 1.3 Update `src/pages/Issue.tsx` to compose the sections
  - Target: page becomes a thin orchestrator (< ~200 lines)
  - Keep routing and top-level layout unchanged

## 2. Issuer management refactor (`src/components/settings/IssuerManager.tsx`)
- [x] 2.1 Extract issuer validation to `src/lib/issuers/validation.ts`
  - Move `validateIssuerForm` logic and error strings
  - Keep validation pure and unit-testable (even if tests are not added yet)
- [x] 2.2 Split UI into components under `src/components/settings/issuers/`
  - `IssuerList.tsx` (list + edit/delete actions)
  - `IssuerForm.tsx` (form fields + submit)
  - `issuer-format.ts` or similar for `formatEnvironment` / `formatIssuerType` helpers
- [x] 2.3 Replace `IssuerManager` with a small composition component
  - Target: `IssuerManager.tsx` becomes a coordinator (~100–150 lines)

## 3. Certificate export modal refactor (`src/components/certificates/CertificateExportModal.tsx`)
- [x] 3.1 Extract destination preference logic to `src/hooks/useExportDestination.ts`
  - Load saved destination and fallback to `downloadDir()`
  - Provide `selectDestination()` and `persistDestination()` helpers
- [x] 3.2 Split modal sections into components under `src/components/certificates/export/`
  - `ExportBundleSelector.tsx`
  - `ExportDestinationPicker.tsx`
  - `PrivateKeyExportWarning.tsx`
  - `ExportResultBanner.tsx`
- [x] 3.3 Keep the IPC call boundary unchanged
  - Continue calling `exportCertificatePem(...)` with the same request mapping

## 4. DNS provider configuration refactor
- [x] 4.1 Split token/credential inputs in `src/components/dns-providers/`
  - Create `ProviderCredentialsFields.tsx` to render Route53 vs token inputs
  - Keep `DnsProviderForm.tsx` focused on layout + wiring
- [x] 4.2 Split `useDnsProviderManager` responsibilities
  - Extract token test into `src/hooks/useDnsProviderTokenTest.ts`
  - (Optional) Extract list/test actions into `useDnsProviderList.ts`
  - Keep public API used by `DnsProvidersPage` stable or adjust with minimal callsite changes

## 5. Conventions + dead-code cleanup
- [x] 5.1 Standardize naming + handler conventions across touched modules
  - Prop handlers: `onX`
  - Local handlers: `handleX`
  - Boolean state: `isX` / `hasX`
- [x] 5.2 Standardize import grouping across touched modules
  - External imports first, blank line, internal imports
- [x] 5.3 Remove dead code
  - Delete unused exports and unreachable branches discovered during refactor
  - Ensure TypeScript strict compile passes without warnings

## 6. Validation
- [x] 6.1 Manual smoke test checklist (requires manual testing)
  - Navigate: Certificates → Issue → Settings → DNS Providers
  - Start issuance (staging issuer) and confirm DNS instructions render
  - Check propagation flow still works (manual + managed paths)
  - Export modal opens for managed certificates and persists destination preference
  - Issuer add/edit/delete works; DNS provider add/edit/test/delete works
- [x] 6.2 Build validation
  - Run `npm run build` and ensure no TypeScript errors


