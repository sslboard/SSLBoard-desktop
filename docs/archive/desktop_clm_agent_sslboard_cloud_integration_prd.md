# Product Requirements Document (PRD)

## Product Name

SSLBoard Desktop CLM Agent

## Document Purpose

This document defines the functional and product requirements for the **SSLBoard Desktop Certificate Lifecycle Management (CLM) Agent** and its optional integration with **sslboard.com cloud services**. It serves as a reference for product design, engineering implementation, and future roadmap decisions.

The goal is to clearly separate:

- A **fully functional, open-source desktop agent** for local certificate issuance and management
- A **paid cloud backend** providing global certificate intelligence, visibility, and compliance features

---

## Problem Statement

Organizations increasingly rely on TLS certificates but face several challenges:

- Certificate issuance is easy; **certificate visibility is not**
- Teams lack a reliable inventory of **all certificates actually in use**, especially those issued outside their tooling
- Existing CLM solutions often require handing over DNS or infrastructure API keys to SaaS platforms
- Security and compliance teams distrust opaque certificate management tools

There is a need for:

- A **local, auditable, privacy-preserving** way to issue and manage certificates
- A **global intelligence layer** that reveals certificates issued elsewhere, historical changes, and risks

---

## Product Vision

Provide a **trustworthy desktop CLM agent** that works entirely locally and integrates optionally with **SSLBoard cloud services** to deliver full certificate lifecycle visibility.

Key principle:

> *Issuance is a commodity. Visibility and intelligence are the product.*

---

## Target Users

- Security engineers
- Infrastructure / platform engineers
- PKI and TLS owners
- Small to mid-sized SaaS companies
- Consultants and auditors
- Developers who want local control over certificate issuance

---

## Non-Goals

The desktop agent will **not**:

- Act as a hosted CA
- Replace enterprise PKI systems
- Handle certificate deployment to production infrastructure (out of scope for v0)
- Enforce licensing or billing logic locally

---

## High-Level Architecture

### Desktop Agent (Open Source)

- Runs locally on user machine
- No mandatory cloud dependency
- Fully functional without login
- Security-critical operations remain local

### SSLBoard Cloud Backend (Paid)

- Hosted service at sslboard.com
- Provides certificate intelligence and enrichment
- Enforces access control and billing

---

## Core Principles

- **Trust first**: All private key operations are open source and auditable
- **Local-first**: No cloud dependency for issuance or renewals
- **Optional cloud**: Login enables additional intelligence, not core functionality
- **Clear boundaries**: Desktop app is a client; backend is the policy engine

---

## Functional Requirements – Desktop Agent (OSS)

### Certificate Issuance

- Support ACME protocol
- Default issuer: Let’s Encrypt (production + staging)
- Support DNS-01 and HTTP-01 challenges
- Support wildcard certificates via DNS-01

### ACME Account Management

- Generate and store ACME account key locally
- Reuse account key for renewals
- Display ACME account metadata

### Key Management

- Generate private keys locally (RSA and ECDSA)
- Store private keys securely using OS key store
- Never transmit private keys outside the machine

### CSR Support

- Import existing CSR
- Generate new CSR locally
- Allow reuse of private keys when renewing

### Certificate Storage

- Store issued certificates locally
- Track metadata: subject, SANs, issuer, validity period, serial number
- Distinguish between:
  - Certificates issued by the agent
  - Certificates imported or discovered externally

### Renewals

- Automatic renewal scheduling
- Manual renewal trigger
- Renewal status visibility

### User Interface

- Desktop UI (Tauri-based)
- List of known certificates
- Expiration warnings
- Renewal status
- Issuer information

### Offline Capability

- All above features must work without any network access to sslboard.com

---

## Functional Requirements – Cloud Integration (Paid)

### Authentication

- Optional login to sslboard.com
- OAuth2 or token-based authentication
- Credentials stored securely in OS keychain

### Certificate Discovery (Core Paid Feature)

- Query sslboard.com for:
  - Certificates discovered via Certificate Transparency
  - Certificates issued by other tools or CAs
- Match discovered certificates to user-owned domains

### Cross-Issuer Inventory

- Display certificates from multiple CAs
- Identify certificates not issued by the desktop agent
- Highlight unmanaged or unknown certificates

### Historical Visibility

- Show historical issuance and expiration events
- Track certificate replacement and rotation over time

### Risk & Hygiene Signals

- Expiring certificates
- Unexpected issuers
- Weak key types or algorithms
- Policy violations (future)

### Alerts & Notifications

- Backend-driven alerts
- Delivered via:
  - Desktop UI
  - Email (future)
  - Webhooks (future)

### Reporting

- Compliance-oriented views
- Exportable summaries (PDF/CSV – future)

---

## Feature Gating Model

- Desktop app contains **no local paywall logic**
- Backend APIs enforce access control
- If an API returns authorization errors:
  - UI displays an informational upgrade prompt

The desktop app must remain fully usable without login.

---

## Security Requirements

- Private keys must never leave the local machine
- Cloud integration must only exchange certificate metadata
- All network communication uses TLS
- Clear separation between trusted (local) and untrusted (remote) data

---

## Open Source Strategy

- Desktop agent codebase is fully open source
- License: permissive (Apache 2.0 or MPL 2.0 recommended)
- Cloud backend remains closed source

The agent must not be crippleware.

---

## UX Principles

- Login is optional and non-intrusive
- Cloud features are framed as **global visibility**, not locked functionality
- Clear distinction between:
  - Local certificates
  - External / discovered certificates

---

## Success Metrics

- Desktop agent adoption
- Number of certificates managed locally
- Conversion rate from local-only users to cloud-connected users
- Retention driven by certificate discovery and alerts

---

## Risks & Mitigations

### Risk: Users do not convert

Mitigation:

- Strong differentiation between local view and global reality
- Clear visualization of "unknown" certificates

### Risk: Security trust erosion

Mitigation:

- Keep all sensitive logic OSS
- Avoid local licensing or obfuscation

---

## Future Considerations (Out of Scope for v0)

- Support for additional ACME CAs (ZeroSSL, Buypass, etc.)
- Private PKI / internal CA support
- Certificate deployment integrations
- Team / multi-user desktop workflows
- Advanced compliance reporting

---

## Summary

The SSLBoard Desktop CLM Agent provides a **local, trustworthy foundation** for certificate issuance and management. SSLBoard cloud services extend this foundation with **global certificate intelligence** that cannot be achieved locally.

The product succeeds by making the local experience genuinely useful while making the value of cloud-based visibility obvious and compelling.

