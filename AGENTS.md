<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:

- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:

- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

## Project Notes for AI Assistants

## Scope and Boundaries

- This is a desktop SSL/TLS issuance and PKI app. The UI is untrusted.
- Secrets (DNS creds, CA keys, ACME account keys) MUST stay in the Rust core.
- Any new capability that shifts architecture or security posture should follow OpenSpec.

## Repository Map

- `src/`: React + TypeScript frontend (Vite).
- `src-tauri/`: Rust core and Tauri shell.
- `docs/`: Product and technical design notes.
- `openspec/`: Specs and change proposals.

## Common Commands

- `npm run dev` for the web UI.
- `npm run tauri dev` for the desktop app.
- `npm run build` for a production build.

## Implementation Notes

- Add new Tauri commands under `src-tauri/src/core/commands` and register in `src-tauri/src/lib.rs`.
- Keep DTOs and IPC boundary types in the Rust core; keep the UI focused on workflows and rendering.
- Prefer minimal, incremental changes and keep behavior aligned with the docs in `docs/`.
