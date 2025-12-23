## ADDED Requirements

### Requirement: Export action for Managed certificates
The UI SHALL provide an “Export…” action for certificates with `source = Managed`.

#### Scenario: Export action is available for Managed certificates
- **WHEN** the user views a certificate with `source = Managed`
- **THEN** the UI SHALL show an “Export…” action

#### Scenario: Export action is not available for External certificates
- **WHEN** the user views a certificate with `source = External`
- **THEN** the UI SHALL NOT show an “Export…” action

### Requirement: PEM export bundles
The system SHALL support exporting certificate material in PEM format, including at least: `cert`, `chain`, and `fullchain`. The default export output SHALL include a standard set of PEM files.

#### Scenario: Export fullchain PEM
- **WHEN** the user selects the `fullchain` bundle option
- **THEN** the system SHALL write a PEM file containing the leaf certificate followed by its chain

#### Scenario: Export chain PEM
- **WHEN** the user selects the `chain` bundle option
- **THEN** the system SHALL write a PEM file containing the issuer chain (excluding the leaf certificate)

#### Scenario: Standard set of certificate PEM files is written
- **WHEN** the user exports certificate material
- **THEN** the system SHALL write a standard set of PEM files (at least `cert.pem`, `chain.pem`, and `fullchain.pem`)

### Requirement: Private key export is optional and guarded
The system SHALL allow exporting the private key only when the certificate has a corresponding managed key reference, and it MUST require explicit user intent to do so.

#### Scenario: Private key export option is disabled when no key exists
- **WHEN** the user exports a Managed certificate that does not have a stored managed key reference (e.g., CSR-imported)
- **THEN** the UI SHALL disable the “include private key” option

#### Scenario: Exporting a private key requires confirmation
- **WHEN** the user enables “include private key”
- **THEN** the UI SHALL display a warning and require a confirmation action before invoking export

### Requirement: Export destination uses a per-certificate folder
The export workflow SHALL accept an output directory and write files under a per-certificate subfolder whose default name is derived from the certificate’s first DNS SAN.

#### Scenario: Default folder name derived from primary DNS name
- **WHEN** the user opens the export modal for a certificate with DNS SANs
- **THEN** the UI SHALL propose a folder name derived from the first DNS SAN

#### Scenario: Multiple SANs still produce a reasonable folder default
- **WHEN** the certificate has multiple DNS SANs
- **THEN** the UI SHALL propose a folder name based on a primary name and MAY include a suffix that indicates additional SANs (e.g., `example.com+2`)

#### Scenario: User can edit the folder name before export
- **WHEN** the user edits the proposed folder name
- **THEN** the export SHALL use the edited folder name for the output path

### Requirement: Export preserves trust boundary
The export workflow MUST be performed in the trusted Rust core, and the UI MUST NOT receive raw certificate private key material over IPC.

#### Scenario: UI requests export via parameters only
- **WHEN** the user performs an export
- **THEN** the UI SHALL send only the certificate id and export options to the Rust core and SHALL receive only success/failure and output paths

### Requirement: Overwrite requires explicit confirmation
The system SHALL prompt before overwriting any existing export files and SHALL proceed only if the user confirms overwrite.

#### Scenario: Existing files trigger overwrite prompt
- **WHEN** the user exports to a folder where one or more target files already exist
- **THEN** the UI SHALL prompt the user to confirm overwriting those files

#### Scenario: User declines overwrite
- **WHEN** the user declines the overwrite prompt
- **THEN** the system SHALL NOT overwrite any files and SHALL return a cancelled/aborted result

### Requirement: Exported files use best-effort restrictive permissions
The Rust core SHALL attempt to write exported files with restrictive permissions appropriate to the operating system (e.g., `0600` on Unix-like systems).

#### Scenario: Best-effort permissions applied
- **WHEN** the Rust core writes `privkey.pem` to disk
- **THEN** it SHALL attempt to apply restrictive permissions and return an error if it cannot ensure a safe write (or otherwise report clearly that permissions could not be restricted)


