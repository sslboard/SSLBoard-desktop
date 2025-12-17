# Change: Step-0 UI shell + shadcn/ui baseline (branding, nav, layout)

## Why
The current UI is the default starter screen and does not reflect SSLBoard’s product identity or provide a stable frame for upcoming workflows (inventory, settings, issuance wizard). Establishing a cohesive UI shell and component system early keeps all later features consistent and pleasant.

## What Changes
- Adopt Tailwind CSS and `shadcn/ui` (latest) as the UI component baseline.
- Create an application shell (sidebar/topbar) with primary navigation and placeholder routes for upcoming features.
- Add minimal SSLBoard branding primitives (app name, logo mark placeholder, color tokens) applied consistently across the shell.
- Replace the starter “greet” UI with the new shell, keeping the UI unprivileged (no secrets, no sensitive logic).

## Impact
- Affected specs: `ui-shell`
- Affected code (planned): `src/` (routing/layout/components/styles), `package.json` (UI deps), Tailwind/shadcn config files


