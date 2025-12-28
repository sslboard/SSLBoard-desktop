# Implementation steps

Below is a **v0 roadmap** you can implement in **testable iterations**, mixing **UI steps** and **Rust/Tauri core steps**, aligned with the architecture in your project docs (untrusted UI, trusted Rust core, modular `issuance/`, `secrets/`, `distribution/`, etc.) and your requirements (LE sandbox, pluggable issuer + DNS adapters, CSR import, CT “external” certs, export, renew).

## Roadmap to v0 (iterative, shippable steps)

### 1) Local data model + inventory foundations (no issuance yet) [Done]

**Goal:** the app can store and display certificates (metadata-only at first).

* **Rust**

  * Create local store (SQLite or similar) for:

    * `CertificateRecord` (id, subjects/SANs, issuer, serial, not_before/not_after, fingerprint, source = `External|Managed`, domain roots, tags)
    * `KeyRecord` (only for managed keys; references into SecretStore)
    * `Order/IssuanceAttempt` history (status, timestamps, logs)
  * Implement `list_certificates`, `get_certificate(id)`.
* **UI**

  * “Certificates” screen: table + details panel.
  * Empty-state UX (import / discover / issue).

**Done when:** you can insert a fake cert record and it shows up, persists across restarts.

---

### 2) Secret storage abstraction (required before real issuance) [Mostly done]

**Goal:** private keys + DNS API tokens never touch the UI.

* **Rust** [Done]

  * Implement `secrets::store::SecretStore` (as per docs), backed by OS secret store (Keychain / Credential Manager / Secret Service).
  * Store/retrieve:

    * ACME account key refs
    * Managed private key refs
    * DNS provider credentials refs
* **UI** [Done]

  * Minimal “Settings → Secrets” page that shows *only references* (no raw secrets), with “add / remove” flows driven by Rust.

**Done when:** UI can add a DNS provider token and later the Rust core can fetch it by ID without exposing it back. [Met]

---

### 3) Issuer interface + Let’s Encrypt **SANDBOX** issuer (first real milestone) [Partial]

**Goal:** pluggable issuer from day 1; sandbox is default.

* **Rust** [Partial]

  * Define an `Issuer` trait (or equivalent) with operations like:

    * `ensure_account()`
    * `begin_order(domains)`
    * `get_challenges(order)`
    * `finalize(order, csr)`
    * `download_certificate(order)`
  * Implement **ACME issuer** configured to Let’s Encrypt **staging/sandbox** endpoint by default.
  * Add `issuer_id` + config in local store.
* **UI** [Done]

  * “Issuer” settings: dropdown (LE Sandbox, later LE Prod, later others).
  * Make it visually obvious when you’re on sandbox (banner/badge).

**Done when:** you can create an ACME account against LE staging and persist the account key ref locally. [Not verified]

---

### 4) DNS challenge engine + pluggable DNS “service” (manual mode first) [Done]

**Goal:** issue certs even if zero DNS API integration exists.

* **Rust** [Done]

  * Define `DnsAdapter` trait: `present_txt(zone, name, value)`, `cleanup_txt(...)`, `check_propagation(...)`.
  * Implement **ManualDNSAdapter**:

    * returns the TXT record instructions
    * “wait + recheck” propagation loop
  * Build “zone mapping” concept: hostname → zone → adapter config.
* **UI** [Done]

  * DNS stepper UI for DNS-01:

    * shows the exact `_acme-challenge` TXT record
    * “I’ve added it” → triggers propagation checks
    * shows progress + errors (NXDOMAIN, wrong TXT, TTL delays)

**Done when:** you can run a full DNS-01 flow with the user manually editing DNS. [Met]

---

### 5) “Issue certificate” flow (generate key OR CSR import) [Partial]

**Goal:** you can obtain a real sandbox certificate end-to-end.

* **UI** [Partial]

  * Wizard:

    1. Enter hostname(s) / SANs
    2. Choose key mode:

       * **Generate new private key locally**
       * **Import CSR file**
    3. DNS challenge (from step 4)
    4. Finalize + success screen
* **Rust** [Partial]

  * CSR validation (`issuance/pki/csr.rs`):

    * parse CSR, extract SANs, validate requested names
  * For “generate key” path: generate keypair locally, store private key in SecretStore, generate CSR internally.
  * Store resulting certificate chain + metadata; mark as `Managed`.

**Done when:** you can issue a LE staging cert for `example.yourdomain.com`, and it appears in the inventory as Managed. [Not verified]

---

### 6) Export for managed certificates (PEM first; PFX later if you want) [Done]

**Goal:** distribution pattern #1 from your functional doc is real.

* **UI** [Done]

  * “Export…” action on a managed cert:

    * PEM bundle options (cert, chain, fullchain)
    * include private key checkbox (disabled for CSR-imported keys you don’t have)
* **Rust** [Done]

  * `distribution/export.rs`:

    * write PEM files securely
    * file permissions best-effort (0600 on Unix)
  * Guard rails:

    * warn before exporting private keys
    * optional “require user presence” gate later (macOS Touch ID, if you choose)

**Done when:** exported files work with nginx/traefik/caddy locally. [Met]

---

### 7) CT discovery integration (external certs) via sslboard.com API [Not started]

**Goal:** “inventory everything”, but distinguish custody.

* **UI** [Not started]

  * "Discover certificates for domain" screen:

    * input: apex domain
    * shows list of certs found in CT
    * user can "Add to inventory"
* **Rust** [Not started]

  * Client for sslboard.com CT query API.
  * Map returned metadata into `CertificateRecord` with `source = External`.
  * Dedup logic: if external fingerprint matches existing managed cert, link/merge.

**Done when:** querying a domain populates external certs in inventory with issuer/serial/NB/NA/SANs. [Not met]

---

### 8) Renewal flows (managed + external) [Not started]

**Goal:** one-button renew with sane behavior.

* **UI** [Not started]

  * "Renew" action with a clear plan preview:

    * External cert: "We will re-issue using these names + issuer settings; private key behavior: new key unless CSR provided."
    * Managed cert: choose

      * reuse existing key (preferred)
      * generate new key
      * use/import CSR
* **Rust** [Not started]

  * Renewal planner:

    * Managed cert: if key exists and is exportable, reuse; else generate.
    * External cert: derive SAN set from metadata; create new order.
  * Persist renewal attempt history + outcome.
  * Update inventory record or create new “version” record (your choice; v0 can keep it simple).

**Done when:** you can renew a managed cert without changing the key, and renew an external cert by reissuing a new managed one. [Not met]

---

### 9) First real DNS provider adapter (Cloudflare is a good v0 pick) [Partial]

**Goal:** prove the adapter system; remove “CLI barrier” for many users.

* **UI** [Partial]

  * “DNS Zones” settings:

    * map `example.com` → Cloudflare adapter → select credential
  * Issuance flow auto-selects adapter based on hostname.
* **Rust** [Not started]

  * Implement `dns/cloudflare.rs` adapter using token from SecretStore.
  * Robustness:

    * handle multiple TXT records
    * cleanup on failure
    * propagation check uses authoritative resolvers + configurable retries

**Done when:** issuance works end-to-end without the user touching their DNS UI (beyond granting token). [Not met]

---

### 10) v0 hardening + “make it hard to shoot yourself” [Not started]

**Goal:** reduce footguns and supportability issues.

* Add:

  * structured audit log (append-only events)
  * strong error surfacing (ACME errors, DNS errors)
  * “Sandbox vs Prod” safety switch (prod requires explicit opt-in)
  * import/export UX polish, confirmations, warnings
* Explicitly defer (per your docs): agent/pull distribution, encrypted relay, Secure Enclave-only keys, full K8s automation.

**Done when:** you can demo: discover (CT) → issue (LE sandbox) → list → export → renew, with DNS manual or Cloudflare.
