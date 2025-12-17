## 1. Implementation
- [ ] 1.1 Define `issuance::dns::DnsAdapter` trait with `present_txt`, `cleanup_txt`, and `check_propagation` (or equivalent) in Rust.
- [ ] 1.2 Implement `ManualDnsAdapter` that returns TXT record instructions and supports a “wait + recheck” propagation loop.
- [ ] 1.3 Define and persist zone mapping: hostname → zone → adapter configuration (including adapter id and credential reference where applicable).
- [ ] 1.4 Add Rust commands to:
  - compute the required `_acme-challenge` TXT record(s)
  - initiate propagation checks and return progress/errors
- [ ] 1.5 Add UI DNS-01 stepper:
  - show exact TXT record name/value and zone context
  - “I’ve added it” triggers propagation check loop
  - render progress and common failure messages (NXDOMAIN, wrong TXT, TTL delays)


