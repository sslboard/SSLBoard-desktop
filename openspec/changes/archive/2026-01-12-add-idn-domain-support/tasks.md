## 1. Implementation
- [x] 1.1 Identify issuance/domain validation flow points that accept domain strings and add IDNA normalization.
- [x] 1.2 Preserve Unicode display values for UI workflows while storing punycode for internal use.
- [x] 1.3 Update ACME order creation and DNS record generation to use punycode.
- [x] 1.4 Make DNS zone suffix matching IDN-aware.
- [x] 1.5 Add Rust unit tests for IDN suffix matching (Unicode input vs punycode zones).
- [x] 1.6 Add UI validation coverage or unit tests if applicable for Unicode entry/display.
