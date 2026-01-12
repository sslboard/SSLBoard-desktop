# Change: Add IDN domain support

## Why
Issuance fails when users enter internationalized domain names (IDNs) with Unicode characters. Supporting IDNs improves usability while keeping DNS automation and ACME requests valid.

## What Changes
- Accept Unicode domain input in the UI and display Unicode labels in workflows and certificate views.
- Normalize and convert IDNs to ASCII (punycode/A-labels) within the Rust core before DNS automation and ACME order creation.
- Store punycode for internal matching and persistence, while preserving the Unicode form for UI display.
- Ensure DNS zone suffix matching is IDN-aware.
- Add unit coverage for IDN suffix matching behavior.

## Impact
- Affected specs: certificate-issuance, dns-configuration
- Affected code: UI domain input/display, Rust IDNA normalization and validation, DNS adapter zone matching
