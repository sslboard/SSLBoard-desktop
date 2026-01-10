## MODIFIED Requirements

### Requirement: Certificate records track issuer association
The system SHALL store the issuer identifier (`issuer_id`) with each Managed certificate record to enable revocation and issuer-specific operations.

#### Scenario: Issuance stores issuer identifier
- **WHEN** a certificate is issued through managed issuance
- **THEN** the certificate record SHALL include the `issuer_id` of the issuer used for issuance

#### Scenario: Discovered certificates have no issuer identifier
- **WHEN** a certificate is discovered from external sources
- **THEN** the certificate record SHALL have `issuer_id` set to NULL

### Requirement: Certificate records track revocation state
The system SHALL store revocation metadata including revocation timestamp and reason for certificates that have been revoked.

#### Scenario: Revoked certificate stores revocation metadata
- **WHEN** a certificate is successfully revoked
- **THEN** the certificate record SHALL be updated with `revoked_at` timestamp and `revocation_reason` if provided

#### Scenario: Non-revoked certificates have null revocation fields
- **WHEN** a certificate has not been revoked
- **THEN** the certificate record SHALL have `revoked_at` and `revocation_reason` set to NULL

## ADDED Requirements

### Requirement: Revocation capability for Managed certificates
The system SHALL provide the ability to revoke Managed certificates that were issued through the application, using either the certificate's private key or the issuer's account key for authentication.

#### Scenario: Revocation succeeds with private key authentication
- **WHEN** a user initiates revocation for a Managed certificate with a valid `issuer_id` and `managed_key_ref`
- **AND** the issuer configuration exists and has a valid account key
- **THEN** the system SHALL authenticate using the certificate's private key
- **AND** submit the revocation request to the ACME CA's revocation endpoint
- **AND** update the certificate record with revocation metadata upon successful revocation

#### Scenario: Revocation succeeds with account key authentication
- **WHEN** a user initiates revocation for a Managed certificate with a valid `issuer_id`
- **AND** the certificate's private key is unavailable but the issuer's account key exists
- **THEN** the system SHALL authenticate using the issuer's account key
- **AND** submit the revocation request to the ACME CA's revocation endpoint
- **AND** update the certificate record with revocation metadata upon successful revocation

#### Scenario: Revocation fails when issuer is missing
- **WHEN** a user attempts to revoke a certificate with a missing or invalid `issuer_id`
- **THEN** the system SHALL reject the revocation request with an error indicating the issuer cannot be determined

#### Scenario: Revocation fails when required keys are unavailable
- **WHEN** a user attempts to revoke a certificate
- **AND** neither the certificate's private key nor the issuer's account key is available
- **THEN** the system SHALL reject the revocation request with an error indicating authentication keys are missing

#### Scenario: Revocation fails for Discovered certificates
- **WHEN** a user attempts to revoke a certificate with source "Discovered"
- **THEN** the system SHALL reject the revocation request with an error indicating only Managed certificates can be revoked

#### Scenario: Revocation fails for already revoked certificates
- **WHEN** a user attempts to revoke a certificate that is already revoked (`revoked_at` is not NULL)
- **THEN** the system SHALL reject the revocation request with an error indicating the certificate is already revoked

### Requirement: UI displays revocation capability and status
The certificate detail UI SHALL display a Revoke button for revocable certificates and show revocation status for revoked certificates.

#### Scenario: Revoke button appears for revocable certificates
- **WHEN** a certificate detail view displays a Managed certificate
- **AND** the certificate has a valid `issuer_id`
- **AND** the certificate is not already revoked
- **AND** at least one authentication key is available (private key or issuer account key)
- **THEN** the UI SHALL display a "Revoke" button alongside the Export button

#### Scenario: Revoke button is hidden for non-revocable certificates
- **WHEN** a certificate detail view displays a certificate that cannot be revoked (Discovered source, missing issuer_id, missing keys, or already revoked)
- **THEN** the UI SHALL NOT display a Revoke button

#### Scenario: Revocation status is displayed
- **WHEN** a certificate detail view displays a revoked certificate
- **THEN** the UI SHALL display the revocation status, revocation date, and revocation reason if available

#### Scenario: Revocation requires confirmation
- **WHEN** a user clicks the Revoke button
- **THEN** the UI SHALL display a confirmation dialog before proceeding with revocation
- **AND** the revocation SHALL only proceed if the user confirms
