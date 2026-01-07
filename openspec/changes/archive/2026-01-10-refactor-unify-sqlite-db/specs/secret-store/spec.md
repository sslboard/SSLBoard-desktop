## RENAMED Requirements

- FROM: `### Requirement: Secrets database strict file permissions`
- TO: `### Requirement: Application database strict file permissions`

## MODIFIED Requirements

### Requirement: Application database strict file permissions

The system SHALL create and maintain a single SQLite database file (`sslboard.sqlite`) that contains both non-secret application metadata and secret-store tables. This unified database file MUST be hardened with strict file permissions (mode 0600 on Unix systems) to prevent unauthorized access.

#### Scenario: Database created with restricted permissions

- **WHEN** the application database file is created for the first time
- **THEN** it SHALL have file permissions set to owner read/write only (0600)

#### Scenario: Permissions corrected on startup

- **WHEN** the application starts and the application database has overly permissive permissions
- **THEN** the system SHALL restrict the permissions to 0600
