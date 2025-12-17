# Desktop SSL Issuance & PKI App – Technical Architecture

## Goals

* Keep **all high-risk secrets local** (DNS API credentials, CA private keys, ACME account keys).
* Provide a **stable, minimal IPC surface** between UI and privileged code.
* Support multiple distribution patterns (manual export, Kubernetes, GitOps, agent/pull) without entangling issuance logic.
* Make the system **auditable, testable, and secure-by-default**.

Non-goals:

* Building a fully centralized SaaS CA.
* Building a full enterprise PKI policy engine.

---

## High-Level System Architecture

The application is split into two trust domains.

```
┌──────────────────────────────────────────────┐
│ UI (TypeScript)                              │
│ - React/Svelte UI                            │
│ - State management                           │
│ - Forms, workflows, validation               │
│ - Minimal logic, no secrets                  │
└───────────────────────┬──────────────────────┘
                        │ IPC (typed commands)
┌───────────────────────▼──────────────────────┐
│ Privileged Core (Rust)                        │
│ - ACME client + DNS-01 orchestration          │
│ - Private CA / CSR signing                    │
│ - Secret storage adapters (Keychain, etc.)    │
│ - Distribution engines (k8s/gitops/agent)     │
│ - Audit log                                  │
└──────────────────────────────────────────────┘

Optional:
┌──────────────────────────────────────────────┐
│ Cloud Companion (optional)                   │
│ - Visibility only (metadata sync)            │
│ - Monitoring + alerts                        │
│ - No custody of secrets                      │
└──────────────────────────────────────────────┘
```

Trust boundary:

* UI is treated as **untrusted**.
* Rust core is **trusted**.

---

## Desktop Shell

Recommended:

* **Tauri** with the system WebView.

Rationale:

* Smaller footprint than Chromium-based shells.
* Clear, explicit capability model.
* Easier to justify security posture for a key-handling tool.

---

## UI Layer

### Stack

* TypeScript + React (or Svelte)
* Vite dev server
* Minimal client-side crypto; no handling of private keys

### Responsibilities

* Collect user intent and parameters
* Display inventory and status
* Orchestrate multi-step workflows (DNS challenges, approvals, distribution)
* Render audit log and errors

### Important UI policy

* UI must never receive:

  * DNS API keys
  * CA private keys
  * Endpoint private keys (in CSR-first mode)

---

## Rust Core Layer

### Responsibilities

1. **Issuance**

   * ACME order/challenge/finalization
   * CSR signing for private PKI
   * Key generation (only when needed)

2. **Secrets management**

   * Store and retrieve credentials
   * Enforce access policies for sensitive keys

3. **Distribution**

   * Render artifacts (bundles, YAML, manifests)
   * Apply updates to targets (k8s, gitops)
   * Support agent/pull endpoints (future)

4. **Audit**

   * Append-only local audit events
   * Optional export

---

## Module Boundaries (Rust)

Suggested crate/module layout inside `src-tauri/src`:

* `core/`

  * `commands.rs` – IPC-exposed commands
  * `types.rs` – request/response DTOs
  * `errors.rs` – error mapping

* `secrets/`

  * `store.rs` – trait for secret storage
  * `keychain.rs` – OS keychain adapter (cross-platform)
  * `macos_access.rs` – macOS-specific access control (Touch ID gating, optional)

* `issuance/`

  * `acme.rs` – ACME flows
  * `dns/` – DNS provider adapters
  * `pki/`

    * `ca.rs` – local CA management (root/intermediate)
    * `csr.rs` – CSR validation and signing
    * `policy.rs` – opinionated defaults (validity, EKU)

* `distribution/`

  * `export.rs` – PEM/PFX bundles
  * `k8s.rs` – Kubernetes secret update and rollout helpers
  * `gitops.rs` – write artifacts for GitOps flows
  * `relay.rs` – encrypted publish/pull (future)
  * `agent.rs` – endpoint pull protocol (future)

* `audit/`

  * `log.rs` – append-only local log

* `storage/`

  * `db.rs` – metadata storage (non-secret)

---

## IPC Design

### Core principles

* Keep the IPC surface **small and stable**.
* Use request/response objects with versioning.
* Never pass secrets across IPC.

### Example IPC (TypeScript)

```ts
import { invoke } from "@tauri-apps/api/core";

export type IssuePublicCertRequest = {
  domains: string[];
  email: string;
  dnsProviderId: string;
  dnsCredentialRef: string; // reference only
};

export type IssueCertResult = {
  certificateId: string;
  fingerprint: string;
  notAfter: string;
};

export async function issuePublicCert(req: IssuePublicCertRequest) {
  return invoke<IssueCertResult>("issue_public_cert", { req });
}
```

### Example IPC (Rust)

```rust
#[derive(serde::Deserialize)]
pub struct IssuePublicCertRequest {
    pub domains: Vec<String>,
    pub email: String,
    pub dns_provider_id: String,
    pub dns_credential_ref: String,
}

#[derive(serde::Serialize)]
pub struct IssueCertResult {
    pub certificate_id: String,
    pub fingerprint: String,
    pub not_after: String,
}

#[tauri::command]
pub async fn issue_public_cert(req: IssuePublicCertRequest) -> Result<IssueCertResult, String> {
    // Load secret by ref inside Rust core
    // Run ACME + DNS-01
    // Store metadata
    Ok(IssueCertResult {
        certificate_id: "cert_123".into(),
        fingerprint: "...".into(),
        not_after: "2030-01-01T00:00:00Z".into(),
    })
}
```

---

## Secrets Storage

### Cross-platform default

* Store most secrets via OS secret store:

  * macOS Keychain
  * Windows Credential Vault
  * Linux Secret Service

Use a storage interface:

* `secrets::store::SecretStore`

Keys and tokens are stored and retrieved only by Rust.

### Touch ID / user presence gating (macOS)

For high-sensitivity keys (private CA root/intermediate), add an access policy layer:

* Require user presence for signing operations
* Optionally prefer Secure Enclave keys for non-exportable CA private keys

This should be implemented as:

* macOS-specific adapter under `secrets/macos_access.rs`
* A policy decision in `issuance/pki/policy.rs`

---

## Metadata Storage (Non-Secret)

Store non-secret state locally for UX and auditability:

* Certificates inventory (id, SANs, issuer, expiry)
* Distribution targets and last applied status
* ACME account identifiers (but not private key material)
* Audit events

Suggested options:

* SQLite (via `rusqlite` or `sqlx`)
* Or plain JSON for v1 (if volume is small)

Important rule:

* Private keys and tokens do not go into SQLite.

---

## Issuance Engines

### Public issuance (ACME)

Flow responsibilities:

* Create order
* Handle DNS-01 challenge creation
* Wait for propagation
* Finalize and fetch certificate

DNS provider drivers live under `issuance/dns/*`.

### Private PKI

Two modes:

1. CSR-first (preferred)

* Endpoint generates key
* App signs CSR
* Distribution carries certificate only

2. Issuer-generated

* App generates key + cert
* Distribution carries key material (more sensitive)

These modes should be explicit in types and UI.

---

## Distribution Engines

### Manual export

* Generate PEM bundles
* Generate PFX for Windows ecosystems
* Provide copy/paste snippets for common targets

### Kubernetes

Approach options:

* Direct apply:

  * Use kubeconfig or token with limited RBAC
  * Update Secret in namespace
  * Trigger rollout or rely on controller reload

* GitOps:

  * Write Kubernetes manifests to a repo directory
  * User’s existing CI/CD applies

Keep Kubernetes logic isolated in `distribution/k8s.rs` and avoid mixing issuance with apply mechanics.

### GitOps

* Output artifacts to a chosen directory
* Optionally support SealedSecrets/SOPS flows

### Encrypted relay (future)

* Endpoint identity (public key)
* Desktop encrypts artifacts to endpoint
* Publish to relay
* Endpoint pulls and decrypts

This keeps cloud blind to plaintext.

---

## Optional Cloud Companion Integration

Design goal:

* Cloud receives only **non-sensitive metadata**.

Possible synced fields:

* fingerprint
* notAfter
* SANs
* issuer name
* deployment status (if user opts in)

Never synced:

* DNS credentials
* private keys
* CA material

API design:

* Desktop signs telemetry payloads with a local app identity key (optional)
* Cloud stores and correlates

---

## Security Controls

### Capability minimization

* Disable shell access unless necessary
* Restrict filesystem scope to app directories
* Restrict network to explicit targets (ACME servers, DNS APIs) where feasible

### Threat model assumptions

* UI is not trusted with secrets
* Local malware as same user is not fully preventable
* User-presence gating protects against silent misuse

### Audit log

* Append-only log for:

  * issuance
  * renewals
  * secret access attempts (high-level)
  * distribution actions
* Exportable for compliance

---

## Development Workflow

### UI hot reload

* UI runs via Vite dev server
* Changes in `.ts/.tsx` should hot reload without Rust rebuild

### Rust rebuild

* Rust recompiles and app restarts only when `src-tauri` changes

Practical workflow:

* Develop UI with a mock backend for fast iteration
* Stabilize IPC DTOs early
* Implement Rust commands incrementally

---

## Packaging and Release

* Code signing and notarization are required for smooth macOS distribution.
* Provide automatic updates only if the security model is clear and auditable.

Release artifacts:

* macOS `.app` / `.dmg`
* Windows installer
* Linux packages as needed

---

## MVP Cut Recommendation

To reduce risk and ship quickly:

1. Public ACME issuance (DNS-01)
2. Local secret storage (keychain)
3. Manual export bundles
4. Kubernetes Secret distribution (optional, but high leverage)
5. Local inventory + audit log

Defer:

* Agent/pull distribution
* Encrypted relay
* Secure Enclave integration (unless required by positioning)

---

## Summary

This architecture:

* Keeps secrets local
* Keeps UI unprivileged
* Separates issuance from distribution
* Supports gradual expansion into automation and fleet distribution
* Maintains a strong security narrative compatible with SSL/TLS tooling and compliance expectations

