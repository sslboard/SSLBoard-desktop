# Desktop SSL Issuance & PKI App

A desktop application concept for issuing, managing, and rotating SSL/TLS certificates while keeping high-risk secrets (DNS API credentials, CA private keys, ACME account keys) on the user’s machine.

## What this is

This repo currently contains design docs for a certificate issuance tool that:

- Issues **public certificates** via ACME (e.g., Let’s Encrypt) using **DNS-01** (DNS credentials stored locally).
- Issues **private certificates** via a simple, intentionally constrained private PKI (root or root+intermediate; server/client certs).
- Treats **distribution** as a first-class problem (manual export, Kubernetes/control-plane integration, GitOps, endpoint pull, encrypted relay).
- Can optionally integrate with a cloud companion for **visibility only** (metadata sync; no secret custody).

## Functional principles

- **Local trust boundary**: sensitive credentials never leave the machine.
- **Human-in-the-loop by default**: explicit user intent for sensitive actions.
- **Separation of concerns**: issuance is local; distribution is explicit/configurable; visibility can be external.
- **Opinionated simplicity**: designed for small-to-medium use; not an enterprise PKI policy engine.

## Key ownership models

- **Endpoint-owned keys (preferred)**: endpoints generate private keys; the app signs CSRs; only certificates are distributed.
- **Issuer-generated keys**: the app generates key+cert; distribution becomes sensitive and must be handled accordingly.

## Architecture (intended)

Two trust domains:

- **UI (TypeScript)**: workflows/forms/state; treated as untrusted; must not handle secrets.
- **Privileged core (Rust)**: issuance, secret storage adapters (Keychain/Vault/Secret Service), distribution engines, audit log.

Recommended desktop shell: **Tauri** (system WebView) to keep the footprint small and capabilities explicit.

Suggested Rust module layout under `src-tauri/src`:

- `core/` (IPC commands, DTOs, errors)
- `secrets/` (secret store trait + OS adapters; optional macOS user-presence gating)
- `issuance/` (ACME + DNS drivers; private PKI CA/CSR/policy)
- `distribution/` (export, k8s, gitops, future relay/agent)
- `audit/` (append-only local audit log)
- `storage/` (non-secret metadata, e.g., SQLite)

## MVP cut (recommended)

1. Public ACME issuance (DNS-01)
2. Local secret storage (OS keychain)
3. Manual export bundles (PEM/PFX)
4. Kubernetes Secret distribution (optional, high leverage)
5. Local inventory + audit log

## Docs

- `docs/functional.md` — product scope, distribution patterns, non-goals.
- `docs/technical.md` — intended architecture, IPC design, storage, security controls.
- `docs/other.md` — suggested additional docs to write (threat model, key lifecycle, workflow narratives, MVP scope, positioning).

