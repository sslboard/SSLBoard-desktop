# Change: Add persistent preferences store

## Why

Users need common UI choices (like the last export destination folder) to persist across sessions so they do not have to reselect them every time.

## What Changes

- Add a general preferences store in the Rust core using local metadata storage.
- Expose typed IPC commands to read and update preferences.
- Persist the last chosen export destination directory as a preference.
- Prefill the export destination with the saved preference or the user’s Downloads folder by default.

## Impact

- Affected specs: `preferences` (new capability), `certificate-export` (uses saved destination)
- Affected code (expected):
  - Rust: new preferences storage module + command(s)
  - UI: export modal uses preference to prefill destination and saves on selection/export

