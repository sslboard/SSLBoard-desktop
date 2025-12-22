## ADDED Requirements

### Requirement: Issuance wizard for managed-key issuance
The UI SHALL provide an issuance wizard that collects domain/SAN inputs for a managed-key issuance path, guides through DNS-01 validation, and displays a completion state.

#### Scenario: Wizard collects inputs and drives DNS-01
- **WHEN** the user starts issuance
- **THEN** the wizard SHALL prompt for domains/SANs, present DNS-01 instructions, and proceed to finalize once DNS is satisfied

### Requirement: Managed-key generation path
The system SHALL support a managed-key issuance path that generates a private key in the trusted Rust core, stores it as a `SecretStore` reference, and uses it to create the CSR without exposing key material to the UI.

#### Scenario: Generated key stays in SecretStore
- **WHEN** the user selects “generate key” for issuance
- **THEN** the system SHALL create the key in Rust, store only a secret ref, generate the CSR internally, and proceed with ACME using that key

### Requirement: DNS-01 execution during issuance
The issuance flow SHALL resolve the DNS adapter mapping for the requested domains (defaulting to the manual adapter), provide the `_acme-challenge` TXT instructions, and poll propagation until success or timeout before finalizing.

#### Scenario: Manual DNS instructions and polling
- **WHEN** the issuance flow reaches DNS-01
- **THEN** the system SHALL present the TXT name/value, allow the user to trigger propagation checks, and only continue to finalize once the expected value is observed (or fail with a clear timeout/error)

### Requirement: Certificate persistence as Managed
The system SHALL finalize the ACME order, download the certificate chain, and store certificate metadata in inventory as `Managed`, including any key reference when generated and noting CSR-import cases without a stored key.

#### Scenario: Issued certificate appears in inventory
- **WHEN** issuance completes successfully
- **THEN** the system SHALL persist the certificate chain and metadata, mark the record as Managed, and include the managed key ref if a generated key was used
