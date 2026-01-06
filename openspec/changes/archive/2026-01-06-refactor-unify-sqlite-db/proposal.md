## Why

The Rust core currently persists state across multiple SQLite files (`issuance.sqlite`, `inventory.sqlite`, `preferences.sqlite`, `secrets.sqlite`). This spreads schema/migration logic across modules, increases duplication (connection setup, PRAGMAs, WAL config), and complicates backup/migration/diagnostics because application state is fragmented.

## What Changes

- Replace the multiple SQLite files with a single application database file (`sslboard.sqlite`) located in the app data directory.
- Apply the existing “secrets database” file hardening rules (strict Unix permissions, Windows hardening best-effort) to the unified database file.
- Centralize SQLite initialization and migrations in shared modules so storage modules only define queries and row mapping.
- Provide a startup migration that imports existing data from legacy databases into the unified database and preserves the old files as `*.bak` backups.

## Impact

- **Data layout**: Users will transition from multiple `.sqlite` files to one. On first run after upgrade, the app migrates/merges data.
- **Security posture**: Secrets ciphertext and non-secret metadata share a single file. The file is hardened, but the separation boundary between secrets and other app state is removed.
- **Maintenance**: One migration runner and one set of PRAGMAs reduces duplicated code and makes future schema changes more consistent.

## Non-Goals

- Changing what data is stored (beyond moving it into the unified DB).
- Changing what secrets cross the IPC boundary (UI remains untrusted; secrets do not cross IPC except during creation flows already allowed by spec).
- Introducing new external dependencies or a full ORM.

## Risks and Mitigations

- **Risk: broader blast radius if DB is corrupted/compromised.**
  - Mitigation: strict file permission hardening continues to be enforced for the unified DB; secret ciphertext remains encrypted under the master key.
  - Mitigation: explicitly constrain export/diagnostics tooling to never include secret ciphertext by default.
- **Risk: migration errors or partial merges.**
  - Mitigation: migration is idempotent and transactional where possible; legacy DB files are preserved as `*.bak` for recovery.
- **Risk: increased contention if all storage shares one mutexed connection.**
  - Mitigation: central DB layer owns connection strategy; stores avoid holding locks longer than needed; follow-up may introduce pooled connections if needed.
