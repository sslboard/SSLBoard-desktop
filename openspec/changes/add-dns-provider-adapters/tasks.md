## 1. Rust Core - Adapters

- [ ] 1.1 Implement Cloudflare adapter (create/delete TXT, optional zone id lookup).
- [ ] 1.2 Implement DigitalOcean adapter (create/delete TXT).
- [ ] 1.3 Implement Route 53 adapter (create/delete TXT with hosted zone id).
- [ ] 1.4 Normalize adapter errors into a consistent error enum for UI.
- [ ] 1.5 Add unit tests for record name/zone mapping and provider request shaping.

## 2. DTOs and Storage

- [ ] 2.1 Add provider-specific config fields to DTOs (Cloudflare zone id, Route 53 hosted zone id).
- [ ] 2.2 Validate config presence per provider type on create/update.

## 3. Test Connection Flow

- [ ] 3.1 Wire new adapters into `dns_provider_test` with create/propagate/cleanup.
- [ ] 3.2 Surface provider error categories to the UI.

## 4. UI Updates

- [ ] 4.1 Add provider-specific input fields in DNS Providers form.
- [ ] 4.2 Show error category and suggestion text on failed tests.
