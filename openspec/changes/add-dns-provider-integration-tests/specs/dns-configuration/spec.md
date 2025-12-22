## ADDED Requirements

### Requirement: DNS provider adapter integration tests

The system SHALL provide integration tests for DNS provider adapters (Cloudflare, DigitalOcean, Route 53) that validate adapter correctness against real provider APIs. Integration tests SHALL be gated behind a feature flag and SHALL validate record name formats, TXT value handling, upsert behavior, and verification logic.

#### Scenario: Integration tests require feature flag
- **WHEN** integration tests are executed
- **THEN** the tests SHALL only compile and run when the `integration-tests` Cargo feature is enabled

#### Scenario: Cloudflare adapter validation
- **WHEN** Cloudflare integration tests run with valid credentials
- **THEN** the tests SHALL validate:
  - Record name format handling (FQDN acceptance)
  - TXT value formatting and quoting behavior
  - Upsert behavior (create new vs update existing records)
  - Verification logic (read-after-write validation)
  - Nested subdomain handling (`_acme-challenge.www.example.com`)
  - Error case handling (invalid token, rate limiting, zone not found)

#### Scenario: DigitalOcean adapter validation
- **WHEN** DigitalOcean integration tests run with valid credentials
- **THEN** the tests SHALL validate:
  - Record name format handling (relative name conversion from FQDN)
  - TXT value formatting and quoting behavior
  - Upsert behavior (create new vs update existing records)
  - Verification logic (read-after-write validation)
  - Nested subdomain handling (`_acme-challenge.www.example.com`)
  - Query parameter format (relative name in URL)
  - Error case handling (invalid token, rate limiting, domain not found)

#### Scenario: Route 53 adapter validation
- **WHEN** Route 53 integration tests run with valid credentials
- **THEN** the tests SHALL validate:
  - Record name format handling (FQDN with trailing dot normalization)
  - TXT value formatting and quoting behavior
  - Upsert behavior (ChangeAction::Upsert correctness)
  - Verification logic (read-after-write validation with name normalization)
  - Nested subdomain handling (`_acme-challenge.www.example.com`)
  - Hosted zone discovery (zone matching logic)
  - Error case handling (invalid credentials, rate limiting, zone not found)

#### Scenario: Test credential management
- **WHEN** integration tests execute
- **THEN** credentials SHALL be loaded from environment variables (e.g., `DNS_TEST_CLOUDFLARE_TOKEN`) and tests SHALL fail gracefully if credentials are not provided

#### Scenario: Test cleanup
- **WHEN** integration tests complete (successfully or with failure)
- **THEN** all created test records SHALL be deleted to prevent test pollution

#### Scenario: CI conditional execution
- **WHEN** integration tests run in CI
- **THEN** tests SHALL execute only when manually triggered, scheduled, or when DNS provider adapter code changes

