## 1. Dependencies and Types

- [x] 1.1 Add DNS provider dependencies (reqwest for Cloudflare/DigitalOcean, aws-sdk-route53 + aws-config for Route 53).
- [x] 1.2 Add error category enum to core types.
- [x] 1.3 Update SecretKind to support Route 53 access key and secret separately.
- [x] 1.4 Update DnsProviderTestResult to include error_category field.

## 2. Rust Core - Adapters

- [x] 2.1 Implement Cloudflare adapter with automatic zone discovery (list zones, match suffix).
- [x] 2.2 Implement DigitalOcean adapter (create/update/delete TXT).
- [x] 2.3 Implement Route 53 adapter with automatic hosted zone discovery (list zones, match suffix).
- [x] 2.4 Ensure adapters upsert TXT records when duplicates exist and verify content after write.
- [x] 2.5 Normalize TXT content to quoted string format before create/update.
- [x] 2.6 Map adapter errors to structured error category enum.
- [x] 2.7 Add unit tests for zone discovery, record name mapping, and error categorization.

## 3. Token Validation

- [x] 3.1 Add token validation command for Cloudflare (verify can list zones).
- [x] 3.2 Add token validation command for Route 53 (verify can list hosted zones).
- [x] 3.3 Add token validation command for DigitalOcean (verify can list domains).

## 4. Secret Management

- [x] 4.1 Update DNS provider create/update to support multiple secrets (Route 53).
- [x] 4.2 Update DNS provider delete to cleanup all associated secrets.
- [x] 4.3 Update storage to track multiple secret refs per provider.

## 5. Test Connection Flow

- [x] 5.1 Wire new adapters into `dns_provider_test` with create/propagate/cleanup.
- [x] 5.2 Return structured error categories in test results.

## 6. UI Updates

- [x] 6.1 Remove zone ID input fields from DNS Providers form.
- [x] 6.2 Add "Test Token" button for Cloudflare/Route 53/DigitalOcean providers.
- [x] 6.3 Show error category and suggestion text on failed tests.
- [x] 6.4 Update Route 53 form to collect access key and secret separately.
