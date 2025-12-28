# SSLBoard Desktop

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A secure desktop application for issuing, managing, and distributing SSL/TLS certificates. SSLBoard keeps sensitive secrets (DNS API credentials, CA private keys, ACME account keys) securely stored on your local machine, ensuring they never leave your device.

![SSLBoard Desktop](docs/SSLBoard-desktop-1280.jpg)

## What is SSLBoard Desktop?

SSLBoard Desktop is an open-source tool designed for developers and DevOps teams to handle certificate lifecycle management with a focus on security and simplicity. It supports both public and private certificate issuance, with robust distribution options and a local-first approach to trust boundaries.

### Key Features

- **Public Certificate Issuance**: Automate SSL/TLS certificates via ACME DNS-01 challenges with integrated DNS providers (Cloudflare, DigitalOcean, AWS Route 53).
- **Private PKI**: Issue private certificates using a constrained PKI system (root or root+intermediate CA, server/client certs).
- **Secure Secret Storage**: Secrets are stored locally using OS keychains (macOS Keychain, Windows Credential Vault, Linux Secret Service) and never transmitted.
- **Certificate Export**: Export certificates in standard PEM formats (cert, chain, fullchain) with optional private key export (guarded by user confirmation).
- **Distribution Options**: Manual export, Kubernetes Secret integration, GitOps support, and future encrypted relay capabilities.
- **Inventory Management**: Local certificate inventory with metadata tracking, filtering, and audit logging.
- **Key Algorithms**: Support for RSA (2048/3072/4096) and ECDSA (P-256/P-384) for managed issuance.
- **UI/UX**: Modern React-based interface with shadcn/ui components, built as a Tauri app for cross-platform desktop support.

### Security Principles

- **Local Trust Boundary**: All sensitive operations occur on-device; no secrets are sent to external services.
- **Human-in-the-Loop**: Explicit user consent required for sensitive actions like private key export.
- **Separation of Concerns**: Issuance, distribution, and visibility are modular and configurable.
- **Opinionated Simplicity**: Tailored for small-to-medium teams; not a full enterprise PKI engine.

### Key Ownership Models

- **Endpoint-Owned Keys (Recommended)**: Endpoints generate private keys; SSLBoard signs CSRs and distributes certificates only.
- **Issuer-Generated Keys**: SSLBoard generates key+certificate pairs; distribution requires careful handling.

## Architecture

SSLBoard uses a two-domain architecture for security:

- **UI Layer (TypeScript/React)**: Handles workflows, forms, and state management. Treated as untrusted and never accesses raw secrets.
- **Core Layer (Rust/Tauri)**: Manages issuance, secret storage, distribution, and audit logging. All privileged operations occur here.

Modules under `src-tauri/src`:

- `core/`: IPC commands, DTOs, error handling
- `secrets/`: OS keychain adapters for secure storage
- `issuance/`: ACME drivers, DNS providers, private PKI
- `distribution/`: Export, Kubernetes, GitOps integrations
- `storage/`: Metadata storage (SQLite)
- `audit/`: Append-only local audit log

## Installation

### Prerequisites

- Node.js (v18+)
- Rust (latest stable)
- Tauri CLI: `npm install -g @tauri-apps/cli`

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/sslboard-desktop.git
   cd sslboard-desktop
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Build and run:
   ```bash
   npm run tauri dev  # Development mode
   npm run tauri build  # Production build
   ```

For detailed setup, see `docs/technical.md`.

## Usage

1. **Configure DNS Providers**: Add API tokens for Cloudflare, DigitalOcean, or AWS Route 53 in Settings.
2. **Set Up Issuers**: Create ACME issuers (e.g., Let's Encrypt staging/production).
3. **Issue Certificates**: Use the Issue page to request certificates with DNS-01 automation.
4. **Manage Inventory**: View, filter, and export certificates from the Certificates page.
5. **Distribute**: Export PEM bundles or integrate with Kubernetes Secrets.

See `docs/functional.md` for workflow narratives and `docs/technical.md` for architecture details.

## Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) before getting started.

- **Issues**: Report bugs or suggest features on GitHub.
- **Pull Requests**: Follow the OpenSpec process for proposals (see `openspec/AGENTS.md`).
- **Code Style**: Rust code follows `rust-code-quality/spec.md`; UI follows `ui-code-quality/spec.md`.

### Development Setup

- Run linting: `npm run lint`
- Run tests: `npm run test` (if available)
- Check type safety: `npm run typecheck`

## License

This project is licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.

Note: While the core is open-source, future paid features (e.g., SSLBoard Cloud integration) may require a commercial license.

## Documentation

- `docs/functional.md` — Product scope, distribution patterns, non-goals
- `docs/technical.md` — Architecture, IPC design, storage, security
- `docs/other.md` — Threat model, key lifecycle, MVP scope
- `openspec/specs/` — Detailed specifications for features

## Roadmap (MVP Focus)

1. Public ACME issuance with DNS-01
2. Local OS keychain secret storage
3. PEM export with guarded private key option
4. Kubernetes Secret distribution
5. Certificate inventory and audit logging

For upcoming features, check `openspec/changes/`.

