# Frontend Notes (src/)

## Stack
- React + TypeScript + Vite.
- Tailwind CSS for styling.

## Boundaries
- Treat the UI as untrusted; do not handle secrets or private key material here.
- All sensitive operations must go through Tauri IPC commands in the Rust core.

## Conventions
- Prefer functional components and hooks.
- Keep view logic in `src/pages/` and reusable UI in `src/components/`.
- Use `src/lib/` for shared helpers and UI utilities.
- Keep imports relative (no path aliases configured).
