# Project Context

## Purpose

`sslboard-desktop` is a **desktop application** for issuing, managing, and rotating SSL/TLS certificates while keeping **high-risk secrets on the user’s machine** (e.g., DNS API credentials, CA private keys, ACME account keys).

Primary goals:

- Keep a **local trust boundary** (no secret custody by a SaaS)
- Enable **public issuance** (ACME / DNS-01) and **simple private PKI** (small root/intermediate hierarchy)
- Treat **certificate distribution** as first-class (export, control-plane integration, GitOps, endpoint pull/relay as future work)
- Maintain **auditability** and a strong security posture (human-in-the-loop for sensitive actions)

## Tech Stack

- **Desktop shell**: Tauri v2 (`src-tauri/`, Rust 2021)
- **UI**: React 19 + TypeScript (strict) (`src/`)
- **Build tooling**: Vite 7 (dev server fixed port `1420` for Tauri)
- **Tauri APIs/plugins**: `@tauri-apps/api` v2, `tauri-plugin-opener` v2
- **Runtime/tooling**: Node/npm (`package-lock.json`)

## Project Conventions

### Code Style

- **TypeScript/React**
  - Prefer **functional components** and hooks.
  - Keep UI code **unprivileged**: do not handle or display raw secrets (DNS API keys, CA private keys, endpoint private keys).
  - Prefer **typed IPC boundaries**: define request/response DTOs in TS and mirror them in Rust.
  - Project is **TypeScript `strict`** with `noUnusedLocals` and `noUnusedParameters` enabled (treat warnings as real issues).
- **Rust**
  - Tauri commands are exposed via `#[tauri::command]`.
  - Prefer **small, explicit modules** and boring, readable code over cleverness (especially around crypto/PKI).
  - Formatting: use `rustfmt` (`cargo fmt`) and idiomatic clippy-friendly patterns.
- **General**
  - Default to **simplicity** (single-purpose modules, minimal abstractions).
  - Avoid introducing new dependencies unless they clearly reduce risk or complexity.

### Architecture Patterns

- **Two trust domains**
  - **UI (TypeScript)**: workflows/forms/state/rendering; treated as **untrusted**.
  - **Privileged core (Rust)**: issuance, secret storage, distribution engines, audit log; treated as **trusted**.
- **IPC design**
  - Keep the IPC surface **small, stable, and typed**.
  - **Never pass secrets across IPC**; UI should pass only **references** (e.g., `dnsCredentialRef`) and parameters.
- **Separation of concerns**
  - **Issuance** is local and security-sensitive.
  - **Distribution** is explicit/configurable and should not be entangled with issuance logic.
  - **Visibility/monitoring** may be external *optionally* (metadata only; no secret custody).
- **Suggested Rust module boundaries** (intended target layout under `src-tauri/src/`)
  - `core/` (commands, DTOs, errors)
  - `secrets/` (SecretStore trait + OS adapters; optional user-presence gating on macOS)
  - `issuance/` (ACME + DNS adapters; private PKI CA/CSR/policy)
  - `distribution/` (export, k8s, gitops; relay/agent are future)
  - `audit/` (append-only local audit log)
  - `storage/` (non-secret metadata, e.g., SQLite or JSON for early versions)

### Testing Strategy

- **Current state**: no dedicated test harness is set up yet (project is still at template-level implementation).
- **Target direction**
  - **Rust**: unit tests in core modules (`cargo test`), plus focused tests for parsing/validation (CSR, SAN rules, policy defaults).
  - **UI**: component/unit tests once workflows solidify (likely `vitest` + React Testing Library), with minimal mocking of IPC.
  - **Security-sensitive logic** should be tested in Rust, not reimplemented in the UI.

### Git Workflow

- **Not formally specified yet**.
- Recommended conventions:
  - Use **feature branches** and PRs for review.
  - For any new capability / behavioral change, prefer an **OpenSpec change proposal** under `openspec/changes/`.
  - Keep commits small and descriptive; **Conventional Commits** are welcome but not required unless enforced later.

## Domain Context

- **Public issuance**: ACME flows (e.g., Let’s Encrypt) with emphasis on **DNS-01** challenges; DNS provider credentials are stored locally.
- **Private PKI (simple)**: local root CA or root + intermediate; issues server and client certs; explicitly *not* enterprise PKI.
- **Distribution is core**: issuance is not useful unless certs reach workloads; supported patterns include:
  - Manual export (PEM/PFX)
  - Control-plane integration (e.g., Kubernetes Secrets / ingress)
  - GitOps distribution (sealed/encrypted artifacts committed and applied by CI/CD)
  - Endpoint pull + encrypted relay publishing (future)
- **Key ownership models**
  - **Endpoint-owned keys (preferred)**: endpoints generate private keys; app signs CSRs; only certificates are distributed.
  - **Issuer-generated keys**: app generates key+cert; distribution becomes highly sensitive.

## Important Constraints

- **Secrets must remain local**: DNS API credentials, CA private keys, ACME account keys must not be sent to any cloud service.
- **Human-in-the-loop by default**: no silent issuance/rotation; sensitive operations require explicit user intent.
- **UI treated as untrusted**: do not move sensitive logic into the UI; keep secret access inside Rust only.
- **Capability minimization**: keep Tauri permissions tight and avoid enabling broad OS capabilities without justification.
- **Product scope**: optimized for small-to-medium use; avoid enterprise PKI complexity (policy engines, multi-tenant CAaaS, hyperscale automation).

## External Dependencies

- **ACME CA endpoints** (planned): Let’s Encrypt and/or other ACME-compatible services.
- **DNS provider APIs** (planned): for DNS-01 automation (credentials stored locally).
- **OS secret stores** (planned): macOS Keychain, Windows Credential Vault, Linux Secret Service.
- **Kubernetes** (optional/planned): write/apply Secrets and rollout helpers.
- **GitOps tooling** (optional/planned): workflows involving encrypted artifacts (e.g., SOPS/SealedSecrets/KMS—implementation TBD).
- **Cloud companion** (optional/future): metadata sync and visibility only (fingerprints, expiry, SANs), with **no secret custody**.
