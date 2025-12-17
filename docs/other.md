Below is the minimal, non-bureaucratic set I’d recommend — with a clear rationale for each.

---

## 1. **Threat Model & Trust Assumptions** (very important for this product)

This is *not* a generic security doc. It’s the one that answers:

* What are we explicitly protecting?
* From whom?
* And what we **accept** as out of scope?

### Why you need it

You’re building a product whose *entire value proposition* is trust boundaries.
Without this doc, features will slowly erode that boundary.

### What it should contain

* Assets:

  * DNS API keys
  * CA private keys
  * ACME account keys
* Actors:

  * Legitimate user
  * Compromised UI
  * Local malware (same user)
  * Remote attacker
  * Cloud service (honest-but-curious)
* Guarantees:

  * “Secrets never leave the machine”
  * “Issuance requires explicit user intent”
* Explicit non-goals:

  * Protecting against a fully compromised OS
  * Defending against malicious root users

This document is what keeps you from accidentally becoming a SaaS CLM.

---

## 2. **Key & Trust Lifecycle Document**

This is different from tech architecture.
It answers **“what happens to keys over time”**.

### Why you need it

PKI products rot when lifecycle rules are implicit.

### What it should describe

* Where each key is generated
* Where it is stored
* How it is protected (Keychain, Touch ID, Secure Enclave)
* When it rotates
* How it is revoked
* What happens on:

  * Device loss
  * User leaving a company
  * CA compromise

This becomes invaluable later when:

* Users ask hard questions
* Auditors get involved
* You integrate with SSLBoard or CAs

---

## 3. **User Workflow Narratives (End-to-End Flows)**

Not wireframes.
**Narratives**.

### Why you need it

You’re designing multi-step, security-sensitive flows.
AI and humans both work better with stories than diagrams.

### Examples

* “Issue a public cert for `example.com` using DNS-01”
* “Create a private CA and issue an mTLS cert”
* “Rotate an expiring cert used by a Kubernetes ingress”
* “Revoke a compromised certificate”

Each should read like:

> Step 1 → Step 2 → Step 3 → Result → Failure modes

This prevents UX from drifting into accidental insecurity.

---

## 4. **MVP Scope & Explicit Non-Features**

This is *crucial* for something this tempting to overbuild.

### Why you need it

Every PKI idea looks reasonable in isolation. Together, they kill products.

### What it should contain

* v1 features
* v1.1 features
* Explicit “not now” list:

  * Full automation
  * Multi-tenant CA
  * Fleet-wide agents
  * Cloud key custody
  * HA issuer clusters

This doc protects your time and sanity.

---

## 5. **Positioning & Comparison Doc (Internal)**

Not marketing copy.
A *thinking tool*.

### Why you need it

You’re implicitly positioning against:

* Venafi / DigiCert CLM
* certbot / lego
* step-ca / Vault
* Cloudflare / ACM

If you don’t write this down, you’ll subconsciously chase all of them.

### What it should cover

* What we do better
* What we intentionally do worse
* Who should *not* use this product
* Why desktop is a feature, not a limitation

This keeps the product sharp.

---

## 6. **AI Usage & Coding Boundaries (Optional but very “you”)**

Given your workflow, this one is surprisingly useful.

### Why you need it

You’ll use AI heavily. This document tells the AI *how*.

### Contents

* Which parts AI may freely generate
* Which parts require human review
* Style constraints (especially for Rust)
* Forbidden patterns (unsafe, clever lifetimes, silent automation)

This reduces entropy over time.

---

## Minimal set (if you want to stay lean)

If I had to cut this down to the **essential trio beyond what you already have**:

1. **Threat Model & Trust Assumptions**
2. **Key & Trust Lifecycle**
3. **User Workflow Narratives**

With:

* Functional doc → *what it does*
* Technical doc → *how it’s built*
* These three → *why it’s safe and coherent*

That’s enough to:

* Build
* Explain
* Defend
* Iterate without losing the soul of the idea


