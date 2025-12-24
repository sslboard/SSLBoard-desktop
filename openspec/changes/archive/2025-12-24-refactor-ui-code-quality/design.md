## Context
This change addresses UI maintainability concerns identified in `/docs/ui-review.md`, primarily oversized components/hooks and inconsistent conventions. It is explicitly a refactor: no new workflows, no new IPC commands, and no changes to the security boundary.

## Goals / Non-Goals

### Goals
- Reduce file size and effective cyclomatic complexity in the largest UI modules by splitting them into focused components and hooks.
- Make “where does logic live?” predictable:
  - Pages compose sections and orchestrate navigation.
  - Components render and delegate through props.
  - Hooks own state machines / async orchestration.
  - `src/lib/` hosts pure helpers (validation, formatting, mapping).
- Standardize naming and import conventions to improve readability and reduce diff noise.
- Preserve behavior and IPC contracts.

### Non-Goals
- No introduction of a new data-fetching framework (e.g., React Query/SWR) in this change.
- No redesign of UI styling or shadcn/ui baseline.
- No changes to Rust core commands, storage schemas, or secret-handling boundaries.
- No new test framework adoption (e.g., Vitest) in this change; use manual smoke testing.

## Decisions

### Decision: Adopt lightweight UI module size guidelines
- **Pages (`src/pages/**`)**: target ≤ ~200 lines after refactor
- **Components (`src/components/**`)**: target ≤ ~150 lines
- **Hooks (`src/hooks/**`)**: target ≤ ~200 lines

Rationale: These thresholds are small enough to keep modules readable, but large enough to avoid over-fragmentation in early-stage UI.

### Decision: Split by responsibilities, not by “type”
Avoid creating “utility” junk drawers. Prefer feature-adjacent folders for cohesive UI logic:
- `src/components/issue/*` for issuance sections
- `src/components/settings/*` for settings sub-features
- `src/hooks/*` for stateful orchestration
- `src/lib/*` for pure helpers

### Decision: Standardize handler naming
- Props: `onX` (e.g., `onSubmit`, `onCancel`, `onSelectIssuer`)
- Local handlers: `handleX`

## Implementation Plan (High-Level)
- **Issue flow**: Extract a “managed issuance” hook that owns the multi-step state machine; split the page into section components.
- **Issuer manager**: Split list + form, extract issuer validation into a helper module.
- **Certificate export modal**: Split into subcomponents and optionally a hook for destination preference loading/persisting.
- **DNS provider configuration**: Split credential input sections and token test UI; reduce `useDnsProviderManager` responsibilities.
- **Conventions cleanup**: Normalize imports and naming; remove dead exports.

## Risks / Trade-offs
- **Risk**: Over-splitting can introduce indirection.
  - **Mitigation**: Prefer 3–6 cohesive subcomponents per feature; avoid creating single-use components unless they remove meaningful complexity.
- **Risk**: Behavior drift during refactors.
  - **Mitigation**: Keep existing props and state transitions; add manual smoke checklist and validate with TypeScript/build.

## Migration Plan
No user-facing migration. This is internal UI-only refactoring.

## Open Questions
- Should we add ESLint + import/order rules to *enforce* conventions, or keep this refactor enforcement-free (TS-only)?
- Do we want a stricter file-size threshold (e.g., 150 lines pages) once the UI matures?


