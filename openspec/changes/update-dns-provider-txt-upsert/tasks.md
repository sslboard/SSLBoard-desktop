## 1. Implementation
- [ ] 1.1 Add TXT content normalization helper for provider adapters (quote wrapping).
- [ ] 1.2 Update DigitalOcean adapter to upsert TXT records on duplicate name and verify content after write.
- [ ] 1.3 Update Route 53 adapter to upsert TXT records on duplicate name and verify content after write.
- [ ] 1.4 Surface verification failures as provider errors in the issuance flow.

## 2. Validation
- [ ] 2.1 Manually validate DigitalOcean TXT upsert and post-write verification.
- [ ] 2.2 Manually validate Route 53 TXT upsert and post-write verification.
