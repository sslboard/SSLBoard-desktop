# Desktop SSL Issuance & PKI App – Functional Overview

## Purpose

The product is a **desktop application** that allows users to issue, manage, and rotate SSL/TLS certificates **without delegating sensitive credentials to a SaaS**. It re-centers trust and control on the user’s machine while still enabling visibility, auditability, and optional cloud intelligence.

The core functional goal is to solve three problems simultaneously:

* Secure certificate issuance
* Simple private PKI when needed
* Safe distribution of certificates to where they are actually used

---

## Core Functional Principles

1. **Local trust boundary**

   * Sensitive credentials never leave the user’s machine
   * The desktop app is the security anchor

2. **Human-in-the-loop by default**

   * Explicit user intent for sensitive actions
   * No silent issuance or rotation

3. **Separation of concerns**

   * Issuance is local
   * Distribution is explicit and configurable
   * Visibility and monitoring can be external

4. **Opinionated simplicity**

   * Designed for small to medium-scale use
   * Avoids enterprise PKI complexity

---

## Functional Scope

### 1. Certificate Issuance

The app supports two issuance modes under a single mental model.

#### Public Certificates

* Uses ACME (e.g. Let’s Encrypt)
* Supports DNS-01 challenges
* DNS API credentials are stored locally
* No DNS credentials are shared with third parties

Typical use cases:

* Public websites
* Customer-facing services
* External endpoints

#### Private Certificates (Simple Private PKI)

* Local root CA or root + intermediate
* Issues server and client certificates
* Suitable for mTLS, internal services, devices

Typical use cases:

* Internal services
* Staging environments
* Corporate tools
* VPN / Wi-Fi / agents

The app is explicitly **not** a full enterprise PKI.

---

### 2. Private PKI Characteristics

The private PKI is intentionally constrained:

* Single root or small hierarchy
* Strong defaults (algorithms, validity)
* Short-lived certificates encouraged
* Clear visibility of what trusts what

Non-goals:

* Massive automation
* Complex policy engines
* Multi-tenant CA-as-a-service

The PKI exists to serve humans and small teams, not fleets at hyperscale.

---

## Certificate Distribution (Critical Problem)

Issuance alone is insufficient. Certificates must reach the systems that use them.

The product treats **distribution as a separate functional concern**, with multiple supported patterns.

---

### Distribution Pattern 1: Manual Export

* Export certificate bundles (PEM, PFX)
* User installs them manually

Characteristics:

* Simple
* No agents
* Human-driven

Limitations:

* Easy to forget rotations
* Not scalable

This is the baseline functionality.

---

### Distribution Pattern 2: Control-Plane Integration

The app can integrate with systems that already act as certificate control planes.

Examples:

* Kubernetes Secrets
* Ingress controllers
* Load balancers

Characteristics:

* No endpoint agents
* Fits modern infrastructure
* Rotation can be automated

Security model:

* Credentials are limited to deployment scope
* Still no DNS key sharing

---

### Distribution Pattern 3: GitOps-Based Distribution

Certificates are distributed via infrastructure-as-code workflows.

Flow:

* Issue certificate locally
* Commit encrypted artifacts or sealed secrets
* CI/CD applies changes

Benefits:

* Auditable
* Rollback-friendly
* Scales well

Tradeoffs:

* Requires infra maturity
* Shifts trust to Git/KMS/SOPS

---

### Distribution Pattern 4: Endpoint Pull Model

Endpoints retrieve certificates themselves.

Flow:

* Endpoint is enrolled once
* Endpoint authenticates (e.g. mTLS)
* Endpoint pulls latest certificate

Benefits:

* Automatic rotation
* Strong security posture
* Good fit for mTLS

Tradeoffs:

* Requires an agent or enrollment step
* Desktop issuer must be reachable or publish artifacts

---

### Distribution Pattern 5: Encrypted Relay Publishing

Used when the desktop issuer should not be directly reachable.

Flow:

* Certificates encrypted to endpoint identity
* Uploaded to a neutral relay
* Endpoint pulls and decrypts

Benefits:

* No cloud key custody
* Desktop can stay offline most of the time

---

## Key Ownership Models

Two distinct models are supported.

### Model A: Endpoint-Owned Keys (Preferred)

* Endpoints generate their own private keys
* Desktop app signs CSRs
* Only certificates are distributed

Advantages:

* Strongest security
* Simplifies distribution
* Private keys never move

### Model B: Issuer-Generated Keys

* Desktop app generates key + cert
* Bundle is distributed securely

Advantages:

* Simpler UX
* Faster onboarding

Tradeoff:

* Distribution becomes sensitive

The product should encourage Model A by default.

---

## Relationship with Cloud Services (Optional)

The desktop app may optionally integrate with a cloud service for **visibility only**.

Possible synced data:

* Certificate fingerprints
* Expiry dates
* SANs
* Issuer metadata

What never leaves the desktop:

* DNS credentials
* Private keys
* CA secrets

This preserves a strong trust narrative.

---

## Target Users

* Security-conscious SMBs
* Auditors and consultants
* Regulated environments
* Teams refusing SaaS DNS access
* Developers needing simple PKI without infrastructure overhead

---

## Explicit Non-Goals

* Full enterprise CLM replacement
* Large-scale autonomous issuance
* Cloud-based CA custody
* Agent-heavy infrastructure

---

## Functional Positioning Summary

This product is:

* A **local trust anchor**
* A **human-controlled certificate issuer**
* A **bridge between issuance and deployment**

It complements, rather than replaces:

* Monitoring platforms
* Cloud visibility tools
* Existing infrastructure workflows

Its value comes from **restoring control and trust**, not

