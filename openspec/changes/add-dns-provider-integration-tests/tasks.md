## 1. Test Framework Setup
- [x] 1.1 Add `integration-tests` feature flag to `Cargo.toml`
- [x] 1.2 Create `src-tauri/tests/integration/dns_providers/` directory structure
- [x] 1.3 Add test utilities module for credential loading, test domain management, and cleanup helpers
- [x] 1.4 Configure test environment variable loading (e.g., `DNS_TEST_CLOUDFLARE_TOKEN`, `DNS_TEST_DIGITALOCEAN_TOKEN`, etc.)

## 2. Cloudflare Integration Tests
- [x] 2.1 Test record name format (verify FQDN handling)
- [x] 2.2 Test TXT value formatting (verify quoting behavior)
- [x] 2.3 Test upsert behavior (create new vs update existing)
- [x] 2.4 Test verification logic (read-after-write validation)
- [x] 2.5 Test nested subdomain handling (`_acme-challenge.www.example.com`)
- [x] 2.6 Test error cases (invalid token, rate limiting, zone not found)

## 3. DigitalOcean Integration Tests
- [ ] 3.1 Test record name format (verify relative name conversion)
- [ ] 3.2 Test TXT value formatting (verify quoting behavior)
- [ ] 3.3 Test upsert behavior (create new vs update existing)
- [ ] 3.4 Test verification logic (read-after-write validation)
- [ ] 3.5 Test nested subdomain handling (`_acme-challenge.www.example.com`)
- [ ] 3.6 Test query parameter format (verify relative name in URL)
- [ ] 3.7 Test error cases (invalid token, rate limiting, domain not found)

## 4. Route 53 Integration Tests
- [ ] 4.1 Test record name format (verify FQDN handling and trailing dot normalization)
- [ ] 4.2 Test TXT value formatting (verify quoting behavior)
- [ ] 4.3 Test upsert behavior (verify ChangeAction::Upsert correctness)
- [ ] 4.4 Test verification logic (read-after-write validation with name normalization)
- [ ] 4.5 Test nested subdomain handling (`_acme-challenge.www.example.com`)
- [ ] 4.6 Test hosted zone discovery (verify zone matching logic)
- [ ] 4.7 Test error cases (invalid credentials, rate limiting, zone not found)

## 5. Documentation and CI
- [ ] 5.1 Document discovered API behaviors in test comments and README
- [ ] 5.2 Add GitHub Actions workflow for conditional integration test execution
- [ ] 5.3 Create test credentials management guide (how to set up test accounts)
- [ ] 5.4 Add test cleanup verification (ensure no test records remain)

## 6. Validation
- [ ] 6.1 Run integration tests against all three providers
- [ ] 6.2 Verify tests catch known issues (record name format, quoting bugs)
- [ ] 6.3 Verify tests pass with corrected implementations
- [ ] 6.4 Document any API behavior discrepancies discovered
