## 1. Implementation

- [x] 1.1 Add a preferences storage table in the local metadata database with upsert semantics.
- [x] 1.2 Define DTOs and Tauri commands to get and set preferences in the Rust core.
- [x] 1.3 Update the export modal to load the saved destination preference and prefill it.
- [x] 1.4 Save the chosen export destination as a preference after selection or successful export.
- [x] 1.5 Default the export destination to the user Downloads folder when no preference exists.

## 2. Tests

- [x] 2.1 Add Rust unit tests for preference upsert and retrieval.
- [x] 2.2 Add a lightweight UI test or manual QA checklist for export destination persistence.
