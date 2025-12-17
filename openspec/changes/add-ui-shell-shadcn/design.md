## Context
The app UI is treated as untrusted and should focus on workflows, presentation, and navigation. A stable shell with a consistent component system reduces future churn and makes later steps (inventory, secrets settings, issuance wizard) feel cohesive.

## Goals / Non-Goals
- Goals:
  - Adopt `shadcn/ui` + Tailwind as the component and styling baseline.
  - Provide a consistent app shell (nav + layout) with placeholder routes.
  - Establish minimal SSLBoard branding in the UI.
- Non-Goals:
  - Implement certificate workflows (inventory, issuance, secrets, DNS) beyond placeholders.
  - Introduce complex theming systems; keep it simple and consistent.

## Decisions
- Decision: Prefer `shadcn/ui` components over bespoke UI to speed iteration and maintain consistency.
- Decision: Keep routing and layout in `src/` with a small shared UI component layer to avoid scattering layout logic.
- Decision: Styling SHOULD follow Tailwind + CSS variables patterns used by shadcn.

## Risks / Trade-offs
- Introducing new UI dependencies increases surface area → Mitigation: keep additions minimal and aligned with shadcn defaults.
- Over-structuring early → Mitigation: start with a simple shell and only add complexity when needed by real workflows.

## Migration Plan
- Replace the starter screen with the shell; preserve any useful dev-only utilities (if needed) as a hidden dev page.

## Open Questions
- Do we want light-only, dark-only, or light/dark toggle for v0?


