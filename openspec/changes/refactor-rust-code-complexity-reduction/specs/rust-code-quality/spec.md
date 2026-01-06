## MODIFIED Requirements
### Requirement: File size limits enforced
Source code files SHALL NOT exceed 400 lines. Files exceeding this limit MUST be split into smaller, focused modules with single responsibilities. Complex functions SHALL be broken down into smaller, focused functions with clear purposes. Mixed responsibilities SHALL be separated into dedicated modules.

#### Scenario: Large command file is split
- **WHEN** a command module exceeds 400 lines
- **THEN** the file SHALL be split into logical sub-modules
- **AND** each sub-module SHALL have a clear, single responsibility
- **AND** the main module SHALL import and re-export functionality

#### Scenario: Complex functions are decomposed
- **WHEN** a function exceeds 50 lines or handles multiple concerns
- **THEN** it SHALL be broken down into smaller, focused functions
- **AND** each function SHALL have a single, clear purpose
- **AND** complex workflows SHALL be orchestrated through dedicated modules

#### Scenario: Mixed responsibilities are separated
- **WHEN** a module contains both high-level orchestration and low-level operations
- **THEN** low-level operations SHALL be extracted to dedicated modules
- **AND** the main module SHALL focus on coordination and error handling
- **AND** clear interfaces SHALL be defined between modules

## ADDED Requirements
### Requirement: Module organization standards
Modules SHALL be organized by functional responsibility with clear separation between high-level orchestration, low-level operations, and cross-cutting concerns. DNS testing logic SHALL be separated from DNS provider abstractions. ACME workflow orchestration SHALL be separated from certificate issuance details. Validation logic SHALL be centralized and reusable.

#### Scenario: DNS concerns properly separated
- **WHEN** implementing DNS provider functionality
- **THEN** DNS provider abstractions SHALL be separate from DNS testing logic
- **AND** DNS cleanup operations SHALL be in dedicated modules
- **AND** DNS validation SHALL be centralized and reusable

#### Scenario: Issuance workflow modularized
- **WHEN** implementing certificate issuance workflows
- **THEN** ACME orchestration SHALL be separate from DNS operations
- **AND** domain validation SHALL be in dedicated functions
- **AND** key generation SHALL be abstracted from workflow logic

#### Scenario: Validation logic centralized
- **WHEN** implementing provider or configuration validation
- **THEN** validation patterns SHALL be extracted to reusable functions
- **AND** error categorization SHALL be consistent across providers
- **AND** validation errors SHALL follow standard formats
