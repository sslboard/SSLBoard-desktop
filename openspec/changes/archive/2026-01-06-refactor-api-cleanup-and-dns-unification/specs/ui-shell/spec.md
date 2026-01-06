## ADDED Requirements
### Requirement: Transparent Vault Management
The UI SHALL NOT require manual vault unlocking or locking by the user as a prerequisite for completing workflows. Secret-dependent operations SHALL automatically trigger backend-level vault access (e.g., via biometric authentication or OS-native prompts) without blocking the user behind a vault-gating overlay.

#### Scenario: User performs secret-dependent action
- **WHEN** the user initiates a DNS provider test or certificate issuance
- **THEN** the UI SHALL NOT display a "vault locked" overlay or require a manual "unlock" button press as a prerequisite for the action
- **AND** the backend SHALL handle any necessary authentication internally

#### Scenario: UI displays vault status without controlling it

- **WHEN** the application shell renders
- **THEN** the UI MAY display a vault status indicator (locked/unlocked)
- **AND** the UI SHALL NOT require the user to toggle this state to proceed with workflows
