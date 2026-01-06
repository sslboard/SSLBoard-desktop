# ui-shell Specification

## Purpose
TBD - created by archiving change add-ui-shell-shadcn. Update Purpose after archive.
## Requirements
### Requirement: UI uses shadcn/ui as the component baseline
The system SHALL adopt `shadcn/ui` (latest) as the primary UI component library and Tailwind CSS as the styling system for the desktop UI.

#### Scenario: Common UI primitives are implemented using shadcn/ui
- **WHEN** the UI renders standard controls (buttons, inputs, dialogs, navigation items)
- **THEN** those controls SHALL use `shadcn/ui` components or composable primitives consistent with shadcn patterns

### Requirement: App shell layout and navigation
The UI SHALL provide a persistent application shell with primary navigation that frames all feature pages.

#### Scenario: Sidebar navigation is available
- **WHEN** the user opens the application
- **THEN** the UI SHALL display a navigation area that allows switching between primary pages

### Requirement: Placeholder routes for upcoming workflows
The UI SHALL include navigable placeholder pages for upcoming core workflows: Certificates, Issue, Discover, and Settings.

#### Scenario: User can navigate to placeholder pages
- **WHEN** the user selects a navigation item
- **THEN** the UI SHALL navigate to the corresponding page and render a placeholder state

### Requirement: Baseline SSLBoard branding
The UI SHALL apply consistent SSLBoard branding across the shell, including application name display and consistent theme tokens (colors and typography).

#### Scenario: Branding is visible in the shell
- **WHEN** the user views the sidebar or topbar
- **THEN** the UI SHALL display the SSLBoard name and a logo mark placeholder (or icon) consistently

### Requirement: UI remains unprivileged
The UI MUST remain untrusted and MUST NOT access, store, or display raw secrets (DNS credentials, private keys, ACME account keys).

#### Scenario: Secrets are not rendered in UI
- **WHEN** the user navigates the UI and views settings pages
- **THEN** the UI SHALL display only non-secret metadata and references, never secret values

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

