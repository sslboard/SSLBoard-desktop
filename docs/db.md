# Database Architecture

SSLBoard uses three separate SQLite databases to organize different types of data with clear separation of concerns. This document describes each database, its tables, schemas, and relationships.

## Overview

The application stores data in three SQLite databases located in the app data directory:

- **macOS**: `~/Library/Application Support/com.sslboard.desktop/`
- **Windows**: `%APPDATA%\com.sslboard.desktop\`
- **Linux**: `~/.local/share/com.sslboard.desktop/`

## Database: `inventory.sqlite`

**Purpose**: Stores certificate inventory and metadata for issued/managed certificates.

**Location**: `{app_data_dir}/inventory.sqlite`

### Table: `certificate_records`

Stores information about SSL/TLS certificates managed by the application.

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
| `source` | TEXT | NOT NULL | How the certificate was obtained (e.g., "acme", "manual") |
| `domain_roots` | TEXT | NOT NULL | Root domains covered by this certificate (JSON array) |
| `tags` | TEXT | NOT NULL | User-defined tags for organization (JSON array) |
| `managed_key_ref` | TEXT | NULL | Reference to the private key secret (points to `secrets.sqlite`) |
| `chain_pem` | TEXT | NULL | Full certificate chain in PEM format |

**Usage**:
- Certificate inventory management
- Certificate lifecycle tracking
- Domain coverage analysis
- Backup and export operations

## Database: `issuance.sqlite`

**Purpose**: Stores ACME (Let's Encrypt) configuration and DNS challenge settings.

**Location**: `{app_data_dir}/issuance.sqlite`

This database contains two tables for ACME certificate issuance configuration.

### Table: `issuer_configs`

Stores ACME account configurations for different Certificate Authorities.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `issuer_id` | TEXT | PRIMARY KEY | Unique identifier for the issuer (e.g., "letsencrypt-staging") |
| `label` | TEXT | NOT NULL | Human-readable name for the issuer |
| `directory_url` | TEXT | NOT NULL | ACME directory URL (e.g., "https://acme-v02.api.letsencrypt.org/directory") |
| `environment` | TEXT | NOT NULL | Environment type ("production", "staging", etc.) |
| `contact_email` | TEXT | NULL | Contact email for ACME account |
| `account_key_ref` | TEXT | NULL | Reference to ACME account private key (points to `secrets.sqlite`) |
| `tos_agreed` | INTEGER | NOT NULL DEFAULT 0 | Whether Terms of Service have been agreed to (0/1) |
| `is_selected` | INTEGER | NOT NULL DEFAULT 0 | Whether this is the currently selected issuer (0/1) |
| `disabled` | INTEGER | NOT NULL DEFAULT 0 | Whether this issuer configuration is disabled (0/1) |
| `created_at` | TEXT | NOT NULL | When the issuer was configured (ISO 8601 datetime) |
| `updated_at` | TEXT | NOT NULL | Last update timestamp (ISO 8601 datetime) |

### Table: `dns_zone_mappings`

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

## Database: `secrets.sqlite`

**Purpose**: Stores metadata and encrypted secret ciphertext. The master encryption key lives in the OS keyring; the database holds only AES-256-GCM ciphertext.

**Location**: `{app_data_dir}/secrets.sqlite`

### Table: `secret_metadata`

Stores non-sensitive metadata about secrets plus encrypted ciphertext.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | Unique identifier for the secret (format: "sec_{uuid}") |
| `kind` | TEXT | NOT NULL | Type of secret (see below) |
| `label` | TEXT | NOT NULL | Human-readable name for the secret |
| `created_at` | TEXT | NOT NULL | When the secret was created (ISO 8601 datetime) |
| `ciphertext` | BLOB | NULL | AES-256-GCM payload stored as `nonce || ciphertext || tag` |

**Secret Kinds**:
- `dns_credential` - DNS provider API credentials
- `acme_account_key` - ACME account private keys
- `managed_private_key` - Private keys for managed certificates

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
dns_zone_mappings.secret_ref → secret_metadata.id (dns_credential)
```

## Architecture Decisions

### Why Separate Databases?

1. **Domain Separation**: Each database serves a distinct purpose
2. **Performance Isolation**: Heavy certificate queries don't impact secret lookups
3. **Independent Evolution**: Each can have separate schema migrations
4. **Selective Operations**: Backup/restore specific data types independently
5. **Size Management**: Certificate inventory can grow large with PEM data

### Thread Safety

Each database store uses `Arc<Mutex<Connection>>` for thread-safe access in the Tauri async runtime.

### Data Persistence

- **Secrets**: Actual secret data stored in OS keyring (Keychain/Credential Manager/Secret Service)
- **Certificates**: Full PEM chains stored in SQLite for backup/export
- **Configuration**: ACME and DNS settings stored in SQLite

## Migration Strategy

Each database includes lightweight schema migration logic to add new columns without breaking existing data. Migrations are applied automatically on application startup.
