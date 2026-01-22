# Database Architecture

SSLBoard uses a unified SQLite database (`sslboard.sqlite`) to store all app data. This document describes the tables, schemas, and relationships.

## Overview

The application stores data in a single SQLite database located in the app data directory:

- **macOS**: `~/Library/Application Support/com.sslboard.desktop/`
- **Windows**: `%APPDATA%\com.sslboard.desktop\`
- **Linux**: `~/.local/share/com.sslboard.desktop/`

Database:
- `sslboard.sqlite` (unified inventory, issuance, preferences, and secrets metadata)

Legacy databases (`inventory.sqlite`, `issuance.sqlite`, `preferences.sqlite`, `secrets.sqlite`) are imported on startup when present and then archived with a `.bak` suffix.

## Table: `certificate_records`

**Purpose**: Stores certificate inventory and metadata for issued/managed certificates.

**Database**: `{app_data_dir}/sslboard.sqlite`

Stores information about SSL/TLS certificates managed or discovered by the application.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | Unique identifier for the certificate record |
| `subjects` | TEXT | NOT NULL | Primary subject(s) of the certificate (JSON array) |
| `sans` | TEXT | NOT NULL | Subject Alternative Names (JSON array) |
| `issuer` | TEXT | NOT NULL | Certificate issuer/CA information |
| `serial` | TEXT | NOT NULL | Certificate serial number |
| `not_before` | TEXT | NOT NULL | Certificate validity start (ISO 8601 datetime) |
| `not_after` | TEXT | NOT NULL | Certificate validity end (ISO 8601 datetime) |
| `fingerprint` | TEXT | NOT NULL | Certificate fingerprint/hash |
| `source` | TEXT | NOT NULL | How the certificate was obtained ("Managed" or "External") |
| `issuer_id` | TEXT | NULL | Issuer configuration id (managed certificates) |
| `domain_roots` | TEXT | NOT NULL | Root domains covered by this certificate (JSON array) |
| `tags` | TEXT | NOT NULL | User-defined tags for organization (JSON array) |
| `managed_key_ref` | TEXT | NULL | Reference to the private key secret (points to `secret_metadata`) |
| `chain_pem` | TEXT | NULL | Full certificate chain in PEM format |
| `key_algorithm` | TEXT | NULL | Key algorithm for managed issuance (rsa/ecdsa) |
| `key_size` | INTEGER | NULL | RSA key size |
| `key_curve` | TEXT | NULL | ECDSA curve (p256/p384) |
| `csr_subject` | TEXT | NULL | CSR subject when issuance used a CSR |
| `csr_sans` | TEXT | NULL | CSR SANs (JSON array) |
| `csr_key_algorithm` | TEXT | NULL | CSR key algorithm |
| `csr_key_size` | INTEGER | NULL | CSR RSA key size |
| `csr_key_curve` | TEXT | NULL | CSR ECDSA curve |
| `csr_source` | TEXT | NULL | CSR source (imported/generated) |
| `renewed_from` | TEXT | NULL | Certificate id this certificate renews |
| `revoked_at` | TEXT | NULL | Revocation timestamp (ISO 8601 datetime) |
| `revocation_reason` | TEXT | NULL | Revocation reason |

**Usage**:
- Certificate inventory management
- Certificate lifecycle tracking
- Domain coverage analysis
- Backup and export operations

## Table: `issuer_configs`

**Purpose**: Stores ACME (Let's Encrypt) configuration and DNS challenge settings.

**Database**: `{app_data_dir}/sslboard.sqlite`

Stores ACME account configurations for different Certificate Authorities.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `issuer_id` | TEXT | PRIMARY KEY | Unique identifier for the issuer (e.g., "letsencrypt-staging") |
| `label` | TEXT | NOT NULL | Human-readable name for the issuer |
| `directory_url` | TEXT | NOT NULL | ACME directory URL (e.g., "https://acme-v02.api.letsencrypt.org/directory") |
| `environment` | TEXT | NOT NULL | Environment type ("production", "staging", etc.) |
| `issuer_type` | TEXT | NOT NULL | Issuer type identifier (e.g., "acme") |
| `params_json` | TEXT | NOT NULL | Issuer-specific parameter payload (JSON) |
| `contact_email` | TEXT | NULL | Contact email for ACME account |
| `account_key_ref` | TEXT | NULL | Reference to ACME account private key (points to `secrets.sqlite`) |
| `tos_agreed` | INTEGER | NOT NULL DEFAULT 0 | Whether Terms of Service have been agreed to (0/1) |
| `is_selected` | INTEGER | NOT NULL DEFAULT 0 | Whether this is the currently selected issuer (0/1) |
| `disabled` | INTEGER | NOT NULL DEFAULT 0 | Whether this issuer configuration is disabled (0/1) |
| `created_at` | TEXT | NOT NULL | When the issuer was configured (ISO 8601 datetime) |
| `updated_at` | TEXT | NOT NULL | Last update timestamp (ISO 8601 datetime) |

### Table: `dns_providers`

Stores DNS provider configurations for automated DNS-01 challenges.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | Unique identifier for the provider |
| `provider_type` | TEXT | NOT NULL | Provider type (e.g., "cloudflare") |
| `label` | TEXT | NOT NULL | Human-readable label |
| `domain_suffixes` | TEXT | NOT NULL | JSON array of domain suffixes |
| `secret_ref` | TEXT | NULL | Reference to provider credentials (points to `secret_metadata`) |
| `config_json` | TEXT | NULL | Provider-specific configuration (JSON) |
| `created_at` | TEXT | NOT NULL | Created timestamp |
| `updated_at` | TEXT | NOT NULL | Last update timestamp |

### Table: `dns_zone_mappings` (legacy)

Maps hostname patterns to DNS zones and their authentication credentials for DNS-01 challenges.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `hostname_pattern` | TEXT | PRIMARY KEY | Pattern to match hostnames (e.g., "*.example.com") |
| `zone` | TEXT | NOT NULL | DNS zone name (e.g., "example.com") |
| `adapter_id` | TEXT | NOT NULL | DNS provider adapter identifier |
| `secret_ref` | TEXT | NULL | Reference to DNS credentials secret (points to `secrets.sqlite`) |
| `created_at` | TEXT | NOT NULL | When the mapping was created (ISO 8601 datetime) |
| `updated_at` | TEXT | NOT NULL | Last update timestamp (ISO 8601 datetime) |

**Usage**:
- ACME account management
- DNS-01 challenge configuration
- Certificate authority selection
- Automated certificate renewal

## Table: `preferences`

**Purpose**: Stores user preferences that should persist across sessions (UI settings, last-used destinations).

**Database**: `{app_data_dir}/sslboard.sqlite`

Key-value store for non-secret user preferences.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `name` | TEXT | PRIMARY KEY | Preference name (unique key) |
| `value` | TEXT | NOT NULL | Preference value |
| `updated_at` | TEXT | NOT NULL | Last update timestamp (ISO 8601 datetime) |

**Usage**:
- Persisting export destination
- Future UI defaults and preferences

## Table: `secret_metadata`

**Purpose**: Stores metadata and encrypted secret ciphertext. The master encryption key lives in the OS keyring; the database holds only AES-256-GCM ciphertext.

**Database**: `{app_data_dir}/sslboard.sqlite`

Stores non-sensitive metadata about secrets plus encrypted ciphertext.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | Unique identifier for the secret (format: "sec_{uuid}") |
| `kind` | TEXT | NOT NULL | Type of secret (see below) |
| `label` | TEXT | NOT NULL | Human-readable name for the secret |
| `created_at` | TEXT | NOT NULL | When the secret was created (ISO 8601 datetime) |
| `ciphertext` | BLOB | NULL | AES-256-GCM payload stored as `nonce || ciphertext || tag` |

**Secret Kinds**:
- `dns_provider_token` - DNS provider API tokens
- `dns_provider_access_key` - DNS provider access key (when providers require key pairs)
- `dns_provider_secret_key` - DNS provider secret key (when providers require key pairs)
- `acme_account_key` - ACME account private keys
- `managed_private_key` - Private keys for managed certificates

Legacy `dns_credential` values are migrated to `dns_provider_token`.

**Usage**:
- Secret inventory management
- UI display of available secrets
- Secret lifecycle tracking
- Reference validation
- AES-256-GCM ciphertext storage for secret values (nonce prepended; master key in OS keyring)

## Database Relationships

The databases are loosely coupled through secret references:

```
certificate_records.managed_key_ref → secret_metadata.id (managed_private_key)
issuer_configs.account_key_ref → secret_metadata.id (acme_account_key)
dns_providers.secret_ref → secret_metadata.id (dns_provider_*)
```

## Architecture Decisions

### Why a Unified Database?

1. **Simpler operations**: One file to manage and backup
2. **Consistent migrations**: Single migration path for all tables
3. **Legacy import support**: Existing split DBs can be merged at startup

### Thread Safety

Each database store uses `Arc<Mutex<Connection>>` for thread-safe access in the Tauri async runtime.

### Data Persistence

- **Secrets**: Actual secret data stored in OS keyring (Keychain/Credential Manager/Secret Service)
- **Certificates**: Full PEM chains stored in SQLite for backup/export
- **Configuration**: ACME and DNS settings stored in SQLite

## Migration Strategy

Each database includes lightweight schema migration logic to add new columns without breaking existing data. Migrations are applied automatically on application startup.
