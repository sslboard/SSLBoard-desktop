## ADDED Requirements
### Requirement: Transparent Vault Management
The UI SHALL NOT require manual vault unlocking or locking by the user. Secret-dependent operations SHALL automatically trigger backend-level vault access (e.g., via biometric authentication or OS-native prompts) without explicit UI-driven state management.

#### Scenario: User performs secret-dependent action
- **WHEN** the user initiates a DNS provider test or certificate issuance
- **THEN** the UI SHALL NOT display a "vault locked" overlay or require a manual "unlock" button press
- **AND** the backend SHALL handle any necessary authentication internally

