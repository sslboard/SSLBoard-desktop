# SSL Certificate Issuance App - Test Scenarios

This document outlines comprehensive test scenarios for the SSL certificate issuance desktop application. These scenarios cover functionality, error handling, security, and edge cases that should be validated during development and before releases.

## Table of Contents

1. [Certificate Lifecycle Management](#1-certificate-lifecycle-management)
2. [Error Handling & Failure Scenarios](#2-error-handling--failure-scenarios)
3. [Certificate Management & Storage](#3-certificate-management--storage)
4. [Security & Privacy Scenarios](#4-security--privacy-scenarios)
5. [Domain & SAN Edge Cases](#5-domain--san-edge-cases)
6. [Issuer Configuration Scenarios](#6-issuer-configuration-scenarios)
7. [DNS Provider Integration Scenarios](#7-dns-provider-integration-scenarios)
8. [Concurrent Operations & Load](#8-concurrent-operations--load)
9. [Integration & Distribution Scenarios](#9-integration--distribution-scenarios)
10. [User Experience Scenarios](#10-user-experience-scenarios)
11. [Platform & Environment Scenarios](#11-platform--environment-scenarios)
12. [Private PKI Scenarios](#12-private-pki-scenarios-future)

---

## 1. Certificate Lifecycle Management

Test scenarios for certificate creation, renewal, and lifecycle management.

### ✅ Already Tested

- Wildcard certificates (`*.domain.com`)
- Multiple SANs across different DNS providers

### 🔄 Planned/To Test

- **Renewal Flow**: Test renewing an expiring certificate (OpenSpec: `add-certificate-renewal-flow`)
  - Pre-populated domains from existing certificate
  - Issuer matching from original certificate
  - Key reuse option vs new key generation
- **Certificate Replacement**: Replace a certificate with different SANs or issuer
- **Expiration Handling**: Test certificates nearing expiry, expired certificates
- **Certificate History/Lineage**: Track renewal chains and certificate relationships

---

## 2. Error Handling & Failure Scenarios

Test how the application handles various failure conditions gracefully.

- **DNS Propagation Failures**: Slow/failed DNS propagation timeouts
- **ACME Errors**: Rate limiting, invalid challenges, account issues
- **DNS Provider Failures**: API authentication errors, network timeouts, provider outages
- **Invalid Domain Scenarios**: Non-existent domains, domains you don't control
- **Mixed Success/Failure**: Some domains succeed, others fail in multi-SAN issuance
- **Network Interruptions**: Connection loss during issuance process
- **Provider API Limits**: Rate limiting, quota exceeded scenarios

---

## 3. Certificate Management & Storage

Test certificate inventory, storage, and management functionality.

- **Export Formats**: PFX/PKCS#12 export, PEM format handling
- **Certificate Inventory**: Search, filter, sort certificates by expiry, issuer, domain
- **Certificate Deletion**: Remove certificates and associated private keys
- **Certificate Discovery**: Import existing certificates from system stores
- **Private Key Management**: Key isolation, secure storage, key reuse scenarios
- **Certificate Metadata**: Tags, notes, custom fields for organization

---

## 4. Security & Privacy Scenarios

Ensure the application's security model is maintained.

- **Credential Isolation**: Ensure DNS credentials can't be accessed from UI
- **Key Security**: Private keys never leave the Rust core, stay in OS keychain
- **Audit Trails**: Certificate issuance history, who/when certificates were issued
- **Secret Management**: Secret reference system, credential rotation
- **Cross-Contamination**: Ensure different issuers/accounts don't mix credentials
- **Clipboard Security**: Sensitive data not leaked to clipboard
- **Log Security**: No secrets or private keys in application logs

---

## 5. Domain & SAN Edge Cases

Test domain validation, normalization, and edge cases.

- **International Domains**: IDNA/punycode domains, Unicode characters
- **Subdomain Wildcards**: `*.sub.example.com` vs `*.example.com`
- **Mixed Domain Types**: Mix of wildcards, regular domains, subdomains in one certificate
- **Domain Normalization**: Leading/trailing dots, case sensitivity, invalid characters
- **SAN Limits**: Test ACME limits on number of SANs per certificate (typically 100)
- **Domain Validation**: Invalid domain formats, reserved domains, localhost
- **IPv4/IPv6 Addresses**: Certificate issuance for IP addresses

---

## 6. Issuer Configuration Scenarios

Test different ACME issuer configurations and account management.

- **Staging vs Production**: Different ACME directories, rate limits, certificate validity
- **Multiple Issuers**: Different ACME providers (Let's Encrypt, ZeroSSL, etc.)
- **Issuer Account Management**: Account creation, key rotation, contact email changes
- **Terms of Service**: ToS acceptance, updates, different provider requirements
- **Issuer Switching**: Change issuers for existing certificate workflows
- **Account Recovery**: Handle lost ACME accounts, key recovery scenarios

---

## 7. DNS Provider Integration Scenarios

Test DNS provider integrations and fallback mechanisms.

- **Provider Overlap**: Multiple providers claiming same domain zones
- **Provider Switching**: Change providers for existing domains
- **Provider Authentication**: Token rotation, expired tokens, permission changes
- **Manual Fallback**: Automatic provider fails → manual DNS instructions
- **Zone Detection**: Automatic zone identification for different domain structures
- **Provider Rate Limits**: Handle API rate limiting gracefully
- **Provider Permissions**: Insufficient permissions for DNS record management

---

## 8. Concurrent Operations & Load

Test system behavior under concurrent load and resource constraints.

- **Parallel Issuances**: Multiple certificates being issued simultaneously
- **DNS Provider Limits**: Rate limiting, concurrent operations per provider
- **Resource Usage**: Memory usage during issuance, cleanup after failures
- **System Stability**: Long-running operations, cancellation handling
- **Database Concurrency**: Multiple operations accessing certificate inventory
- **Network Resource Management**: Connection pooling, timeout handling

---

## 9. Integration & Distribution Scenarios

Test certificate distribution and system integration.

- **Certificate Distribution**: Export certificates for different systems
- **Export Destination Persistence**: Choose an export folder, close the modal, reopen it, and confirm the previous destination is prefilled (fallback to Downloads when missing)
- **System Integration**: Kubernetes secrets, load balancers, web servers
- **GitOps Workflow**: Certificate artifacts for infrastructure-as-code
- **Backup/Restore**: App data backup with certificates and secrets
- **Third-party Tools**: Integration with monitoring, alerting systems
- **Certificate Deployment**: Automated deployment to target systems

---

## 10. User Experience Scenarios

Test user workflows, error messages, and interaction patterns.

- **Workflow Continuity**: Save/resume interrupted issuance processes
- **Progress Tracking**: Real-time status updates during issuance
- **Error Recovery**: Clear error messages, actionable recovery steps
- **Configuration Validation**: Pre-flight checks before attempting issuance
- **User Guidance**: Helpful hints, tooltips, and contextual help
- **Accessibility**: Screen reader support, keyboard navigation
- **Internationalization**: Multi-language support, locale handling

---

## 11. Platform & Environment Scenarios

Test cross-platform compatibility and environmental factors.

- **Cross-Platform**: macOS, Windows, Linux behavior differences
- **Network Conditions**: Offline operation, proxy environments, DNSSEC
- **File System**: Permissions, disk space, special characters in paths
- **OS Integration**: Keychain access, system certificate stores
- **System Resources**: Low memory, limited disk space scenarios
- **Security Policies**: Corporate firewalls, antivirus interference

---

## 12. Private PKI Scenarios (Future)

Test private certificate authority functionality when implemented.

- **Root CA Issuance**: Generate and manage private root certificates
- **Intermediate CA**: Certificate chains and trust hierarchies
- **Client Certificates**: mTLS scenarios, device certificates
- **Revocation**: CRLs, OCSP, certificate revocation workflows
- **Trust Stores**: Managing private CA trust in different systems
- **Certificate Templates**: Custom certificate profiles and constraints

---

## Testing Priorities

### High Priority (Test Before Major Releases)

1. Error handling and failure scenarios
2. Security and privacy scenarios
3. Certificate lifecycle management
4. DNS provider integration scenarios

### Medium Priority (Test During Development Cycles)

1. Domain and SAN edge cases
2. Issuer configuration scenarios
3. User experience scenarios
4. Platform and environment scenarios

### Low Priority (Test During Feature Development)

1. Concurrent operations and load
2. Integration and distribution scenarios
3. Private PKI scenarios (when implemented)

## Testing Methodology

### Manual Testing

- **Exploratory Testing**: Try unexpected combinations of features
- **Edge Case Testing**: Test boundary conditions and unusual inputs
- **Error Injection**: Simulate network failures, API errors, disk full conditions

### Automated Testing

- **Unit Tests**: Individual component functionality
- **Integration Tests**: End-to-end certificate issuance workflows
- **API Testing**: DNS provider integrations, ACME interactions
- **UI Testing**: User interface workflows and error states

### Performance Testing

- **Load Testing**: Multiple concurrent certificate issuances
- **Resource Testing**: Memory usage, disk I/O, network bandwidth
- **Stability Testing**: Long-running operations, memory leaks

## Success Criteria

A test scenario passes when:

- Expected functionality works as designed
- Error conditions are handled gracefully with clear user feedback
- Security boundaries are maintained (secrets stay in Rust core)
- User experience is smooth and intuitive
- Cross-platform compatibility is maintained
- Performance meets acceptable thresholds

---

*Last Updated: December 2025*
*Based on current app functionality and planned OpenSpec changes*
