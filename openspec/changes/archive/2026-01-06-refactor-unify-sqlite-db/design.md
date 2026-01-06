# Design: Unified SQLite database

## Overview

Move all SQLite-backed persistence in `src-tauri` to a single database file in the app data directory (`sslboard.sqlite`). The unified DB contains both:
- non-secret metadata (inventory, issuer config, DNS provider config, preferences)
- secret metadata and encrypted secret ciphertext (as already specified by Secret Store)

The database file is hardened using the same rules previously applied to `secrets.sqlite`.

## Database initialization

Introduce a shared initializer responsible for:
- resolving `app_data_dir`
- opening SQLite with safe flags
- applying PRAGMAs (WAL, foreign keys)
- enforcing file hardening on the unified DB file

## Migrations

Centralize schema creation and migrations in one module that:
- creates all tables if missing
- applies incremental column/table migrations
- performs any backfills required to keep older records compatible

## Legacy database import

On startup:
1. Open/initialize the unified DB and run migrations.
2. Detect legacy DB files:
   - `issuance.sqlite`
   - `inventory.sqlite`
   - `preferences.sqlite`
   - `secrets.sqlite`
3. For each legacy DB, import relevant tables/rows into the unified DB.
4. Preserve legacy DB files by renaming to `*.bak` (or leaving them intact; final behavior confirmed in tasks).

The import is designed to be:
- idempotent (safe to retry)
- best-effort (continues even if one legacy DB is missing)
- transactional per legacy DB where feasible

## Export/diagnostics constraints

Any “export app data” or diagnostics feature MUST:
- exclude secret ciphertext by default
- include secret data only behind explicit, user-confirmed intent
