## Context

DNS provider adapters implement TXT record creation, update, and deletion for ACME DNS-01 challenges. The implementations have several uncertainties:

- **Record name formats**: DigitalOcean expects relative names, Route 53 may require trailing dots, Cloudflare uses FQDNs
- **TXT value quoting**: Some providers require quotes, others add them automatically
- **API response formats**: Providers may return values with different quoting or name normalization
- **Upsert behavior**: Different providers handle duplicate records differently

These uncertainties can only be resolved by testing against real provider APIs. Unit tests cannot validate actual API behavior.

## Goals / Non-Goals

- Goals:
  - Validate actual API behavior for record name formats, TXT value handling, and response formats
  - Catch implementation bugs before production deployment
  - Document discovered API behaviors for future reference
  - Enable regression testing when adapter code changes
  - Provide confidence in adapter correctness
- Non-Goals:
  - Performance benchmarking (focus on correctness, not speed)
  - Load testing (not needed for DNS-01 challenge use case)
  - Multi-provider comparison testing (test each provider independently)
  - Testing provider API reliability (focus on our adapter correctness)

## Decisions

### Decision: Feature flag gating

Integration tests SHALL be gated behind a `integration-tests` Cargo feature flag to avoid requiring credentials for normal builds and tests.

**Rationale**: Integration tests require real API credentials and test domains. Not all developers need to run them, and CI should run them conditionally.

**Alternatives considered**:
- Always require credentials: Rejected - blocks developers without test accounts
- Separate test binary: Considered - feature flag is simpler and allows conditional compilation

### Decision: Environment variable credential loading

Integration tests SHALL load credentials from environment variables (e.g., `DNS_TEST_CLOUDFLARE_TOKEN`) rather than config files or hardcoded values.

**Rationale**: Environment variables are standard for CI/CD, avoid committing secrets, and allow per-run configuration.

**Alternatives considered**:
- Config file: Rejected - risk of committing secrets
- Hardcoded test credentials: Rejected - security risk

### Decision: Test domain management

Each provider SHALL use a dedicated test domain (or subdomain) configured in the test account. Tests SHALL clean up all created records after execution.

**Rationale**: Isolates test data, prevents pollution of production domains, and ensures test reproducibility.

**Alternatives considered**:
- Use production domains: Rejected - risk of affecting real DNS records
- Manual domain setup: Considered - automated cleanup is safer

### Decision: Test structure per provider

Each provider SHALL have a dedicated test module (`cloudflare_test.rs`, `digitalocean_test.rs`, `route53_test.rs`) with shared utilities in `test_utils.rs`.

**Rationale**: Clear organization, allows testing providers independently, and enables conditional compilation per provider.

**Alternatives considered**:
- Single test file: Rejected - too large and harder to maintain
- Test per adapter method: Considered - current structure balances granularity and organization

### Decision: Verification-focused test cases

Tests SHALL focus on verifying adapter correctness (name formats, value handling, upsert behavior) rather than testing provider API features.

**Rationale**: We're testing our adapter implementation, not the provider APIs themselves. Provider API correctness is assumed.

**Alternatives considered**:
- Test provider API features: Rejected - out of scope, providers document their own APIs

### Decision: Conditional CI execution

Integration tests SHALL run in CI only when:
- Manually triggered via workflow dispatch
- Scheduled (e.g., nightly)
- On changes to DNS provider adapter code

**Rationale**: Reduces CI costs, avoids rate limiting, and focuses execution when relevant.

**Alternatives considered**:
- Run on every PR: Rejected - too expensive and may hit rate limits
- Never run in CI: Rejected - defeats purpose of automated testing

## Risks / Trade-offs

- **Risk**: Test credentials exposure
  - **Mitigation**: Use environment variables, never commit credentials, use GitHub secrets in CI

- **Risk**: Test domain costs
  - **Mitigation**: Use free tier domains or subdomains, minimize test execution frequency

- **Risk**: Rate limiting during test execution
  - **Mitigation**: Add delays between tests, use conditional CI execution, handle rate limit errors gracefully

- **Risk**: Test pollution (records not cleaned up)
  - **Mitigation**: Always clean up in test teardown, verify cleanup in CI, use unique test record names with timestamps

- **Trade-off**: Test execution time vs coverage
  - **Decision**: Prioritize correctness validation over speed; accept longer test runs for comprehensive coverage

## Migration Plan

1. Add feature flag and test directory structure
2. Create test utilities for credential loading and cleanup
3. Implement Cloudflare integration tests
4. Implement DigitalOcean integration tests
5. Implement Route 53 integration tests
6. Add CI configuration for conditional execution
7. Document test setup and discovered behaviors
8. Run tests and validate they catch known issues

## Open Questions

- Should we test error cases (invalid credentials, rate limits) or focus only on success paths?
  - **Decision**: Include error cases to validate error handling and categorization

- How should we handle test failures due to provider API changes?
  - **Decision**: Document as known issues, update adapters if API changes are breaking

- Should integration tests be part of the release process?
  - **Decision**: Yes, but manual trigger to avoid blocking releases on provider API issues

