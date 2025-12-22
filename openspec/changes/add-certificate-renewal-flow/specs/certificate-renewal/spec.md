## ADDED Requirements

### Requirement: Renew action navigates to Issue page with pre-filled data

The UI SHALL provide a "Renew" action for existing certificates that navigates to the Issue page with pre-populated domains/SANs, an issuer hint for matching, and an optional key reference for reuse.

#### Scenario: User initiates renewal from certificate detail

- **WHEN** the user clicks "Renew" on a certificate detail view
- **THEN** the system SHALL navigate to the Issue page with the certificate's SANs pre-filled in the domains input and the original issuer label provided as a hint for auto-selection

#### Scenario: Managed certificate renewal offers key reuse option

- **WHEN** the user initiates renewal for a Managed certificate with a `managed_key_ref`
- **THEN** the Issue page SHALL display an option to reuse the existing private key

### Requirement: Issuer matching during renewal

The system SHALL attempt to auto-select an issuer that matches the original certificate's issuer label or type when pre-fill data includes an issuer hint.

#### Scenario: Issuer hint matches an available issuer

- **WHEN** the Issue page receives an issuer hint (e.g., "Let's Encrypt (Staging)")
- **AND** an enabled issuer with a matching label exists
- **THEN** the system SHALL auto-select that issuer

#### Scenario: Issuer hint does not match any available issuer

- **WHEN** the Issue page receives an issuer hint that does not match any enabled issuer
- **THEN** the system SHALL fall back to the default issuer selection (first enabled issuer) and MAY display a notice that the original issuer was not found

### Requirement: Key reuse mode for managed issuance

The system SHALL support an optional key reuse mode where an existing managed private key reference is used instead of generating a new key for the CSR.

#### Scenario: Key reuse succeeds with valid reference

- **WHEN** the user selects key reuse and provides a valid `managed_key_ref`
- **AND** the referenced key exists in the SecretStore
- **THEN** the system SHALL use that key to generate the CSR and proceed with ACME issuance

#### Scenario: Key reuse fails with invalid reference

- **WHEN** the user selects key reuse and provides a `managed_key_ref` that does not exist
- **THEN** the system SHALL return a clear error and SHALL NOT proceed with issuance

### Requirement: Renewal lineage tracking

The system SHALL track renewal lineage by storing a reference from the new certificate to the certificate it renews, enabling users to view renewal history.

#### Scenario: Completed renewal records lineage

- **WHEN** a certificate is issued via the renewal flow with a `renewing_cert_id` provided
- **THEN** the new certificate record SHALL store a `renewed_from` field referencing the original certificate's ID

#### Scenario: Viewing renewal history

- **WHEN** the user views a certificate that has a `renewed_from` reference
- **THEN** the UI SHALL display or allow navigation to the predecessor certificate

