## 1. Spec

- [x] 1.1 Use unified DB filename `sslboard.sqlite`
- [x] 1.2 Preserve legacy DBs as `*.bak` after import

## 2. Rust core implementation

- [x] 2.1 Add shared DB initializer module (path resolution, PRAGMAs, file hardening)
- [x] 2.2 Add shared migrations module for all tables/columns/backfills
- [x] 2.3 Update all stores to use the shared DB handle
- [x] 2.4 Implement one-time import/migration from legacy DB files into unified DB
- [x] 2.5 Ensure migration is idempotent and preserves backups

## 3. Safety and regression

- [x] 3.1 Verify secrets never cross IPC boundary during normal operations
- [x] 3.2 Add/update targeted Rust tests for: migrations, legacy import, and permission hardening behavior

## 4. Cleanup

- [x] 4.1 Remove dead code paths for legacy DB filenames after migration is stable
- [x] 4.2 Update developer docs if they mention old DB filenames
