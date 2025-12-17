## ADDED Requirements

### Requirement: Local certificate inventory storage
The system SHALL persist a local inventory of certificate records containing **metadata only**, and it MUST NOT store private keys or raw credentials in this inventory store.

#### Scenario: Inventory persists across restarts
- **WHEN** a certificate record is created in the local inventory
- **THEN** the record SHALL be present after the application is restarted

#### Scenario: Inventory contains only non-secret fields
- **WHEN** a certificate record is stored
- **THEN** the stored fields SHALL be limited to non-secret metadata (e.g., SANs, issuer, serial, validity, fingerprint, source, tags)

### Requirement: Certificate record shape
The system SHALL represent each inventory entry as a `CertificateRecord` containing, at minimum: `id`, `subjects`/`sans`, `issuer`, `serial`, `not_before`, `not_after`, `fingerprint`, `source` (`External` or `Managed`), `domain_roots`, and `tags`.

#### Scenario: Managed and external sources are distinguishable
- **WHEN** a record is created with `source = Managed`
- **THEN** the UI SHALL be able to distinguish it from `source = External` without requiring any secret data

### Requirement: Inventory read APIs
The system SHALL provide read APIs to the UI for:
- `list_certificates` returning a list of `CertificateRecord` summaries
- `get_certificate(id)` returning a full `CertificateRecord` for a single id

#### Scenario: Listing certificates on an empty store
- **WHEN** `list_certificates` is called and no records exist
- **THEN** the system SHALL return an empty list without error

#### Scenario: Getting a non-existent certificate id
- **WHEN** `get_certificate(id)` is called for an unknown id
- **THEN** the system SHALL return a not-found error suitable for user display

### Requirement: UI certificate inventory screen
The UI SHALL provide a “Certificates” screen with a table of inventory entries and a details panel for a selected entry, including an empty-state that guides the user toward import/discover/issue flows.

#### Scenario: Empty-state UX for first run
- **WHEN** the user opens the “Certificates” screen and the inventory is empty
- **THEN** the UI SHALL show an empty-state with clear next actions


