## ADDED Requirements
### Requirement: Persistent background task visibility
The UI SHALL provide a way for the user to monitor active certificate issuance tasks even after navigating away from the Issue page.

#### Scenario: User navigates away from active issuance
- **WHEN** a certificate issuance task is running in the background
- **AND** the user navigates to the Certificates or Settings page
- **THEN** the UI SHALL maintain a background listener for the task
- **AND** SHOULD provide a visual indicator (e.g., in the sidebar or topbar) that an issuance is in progress

