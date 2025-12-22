## 1. Dependencies and Types

- [ ] 1.1 Add Rust SDK dependencies (aws-sdk-route53, aws-config, cloudflare, digitalocean).
- [ ] 1.2 Add error category enum to core types.
- [ ] 1.3 Update SecretKind to support Route 53 access key and secret separately.
- [ ] 1.4 Update DnsProviderTestResult to include error_category field.

## 2. Rust Core - Adapters

- [ ] 2.1 Implement Cloudflare adapter with automatic zone discovery (list zones, match suffix).
- [ ] 2.2 Implement DigitalOcean adapter (create/delete TXT).
- [ ] 2.3 Implement Route 53 adapter with automatic hosted zone discovery (list zones, match suffix).
- [ ] 2.4 Map adapter errors to structured error category enum.
- [ ] 2.5 Add unit tests for zone discovery, record name mapping, and error categorization.

## 3. Token Validation

- [ ] 3.1 Add token validation command for Cloudflare (verify can list zones).
- [ ] 3.2 Add token validation command for Route 53 (verify can list hosted zones).
- [ ] 3.3 Add token validation command for DigitalOcean (verify can list domains).

## 4. Secret Management

- [ ] 4.1 Update DNS provider create/update to support multiple secrets (Route 53).
- [ ] 4.2 Update DNS provider delete to cleanup all associated secrets.
- [ ] 4.3 Update storage to track multiple secret refs per provider.

## 5. Test Connection Flow

- [ ] 5.1 Wire new adapters into `dns_provider_test` with create/propagate/cleanup.
- [ ] 5.2 Return structured error categories in test results.

## 6. UI Updates

- [ ] 6.1 Remove zone ID input fields from DNS Providers form.
- [ ] 6.2 Add "Test Token" button for Cloudflare/Route 53/DigitalOcean providers.
- [ ] 6.3 Show error category and suggestion text on failed tests.
- [ ] 6.4 Update Route 53 form to collect access key and secret separately.
