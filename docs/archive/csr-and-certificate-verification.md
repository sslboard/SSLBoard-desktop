# CSR Creation and Certificate Verification Guide

This guide explains how to create a Certificate Signing Request (CSR) with Subject Alternative Names (SAN) using OpenSSL, and how to verify that a certificate matches its private key.

## Creating a CSR with SAN

There are two methods to create a CSR with Subject Alternative Names. Method 1 (using a config file) is recommended as it works with all OpenSSL versions and is more flexible.

### Method 1: Using a Config File (Recommended)

1. **Create a config file** `csr.conf`:

```ini
[req]
default_bits = 2048
prompt = no
default_md = sha256
distinguished_name = dn
req_extensions = v3_req

[dn]
CN=example.com

[v3_req]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = example.com
DNS.2 = www.example.com
DNS.3 = mail.example.com
```

Replace `example.com` with your domain and add additional DNS entries in the `[alt_names]` section as needed.

2. **Generate the private key and CSR**:

```bash
openssl req -new -newkey rsa:2048 -nodes \
  -keyout example.com.key \
  -out example.com.csr \
  -config csr.conf
```

This creates:
- `example.com.key` - Your private key (keep this secure!)
- `example.com.csr` - The certificate signing request to submit to a CA

### Method 2: Command-Line Only

For OpenSSL 1.1.1 and later, you can use the `-addext` flag:

```bash
openssl req -new -newkey rsa:2048 -nodes \
  -keyout example.com.key \
  -out example.com.csr \
  -subj "/CN=example.com" \
  -addext "subjectAltName=DNS:example.com,DNS:www.example.com"
```

**Note:** Method 2 requires OpenSSL 1.1.1+. For older versions, use Method 1.

### Verify the CSR

Before submitting your CSR to a CA, verify it contains the expected SAN entries:

```bash
openssl req -in example.com.csr -text -noout
```

Look for the "X509v3 Subject Alternative Name" section to confirm all your DNS entries are present.

## Verifying a Certificate

After receiving a certificate from a CA, verify that it matches your private key and check its validity.

### Check Certificate Matches Private Key

The most important verification is ensuring your certificate was issued for the private key you generated. This is done by comparing the modulus (public key component) of both:

**Quick verification:**
```bash
# Get modulus hash from certificate
openssl x509 -noout -modulus -in certificate.pem | openssl md5

# Get modulus hash from private key
openssl rsa -noout -modulus -in private.key | openssl md5
```

**Both MD5 hashes must match.** If they don't, the certificate and private key don't belong together.

**One-liner to check both at once:**
```bash
openssl x509 -noout -modulus -in certificate.pem | openssl md5 && \
openssl rsa -noout -modulus -in private.key | openssl md5
```

### Check Certificate Details

**View full certificate information:**
```bash
openssl x509 -in certificate.pem -text -noout
```

This shows:
- Subject (CN, Organization, etc.)
- Issuer
- Validity dates
- Subject Alternative Names (SAN)
- Extensions
- Public key information

**Check validity dates:**
```bash
openssl x509 -in certificate.pem -noout -dates
```

Output shows:
- `notBefore` - Certificate valid from this date
- `notAfter` - Certificate expires on this date

**Check subject and issuer:**
```bash
openssl x509 -in certificate.pem -noout -subject -issuer
```

**Check Subject Alternative Names:**
```bash
openssl x509 -in certificate.pem -text -noout | grep -A 2 "Subject Alternative Name"
```

### Verify Certificate Chain

If you have the certificate chain (fullchain.pem), verify the certificate validates against it:

```bash
openssl verify -CAfile fullchain.pem certificate.pem
```

**Note:** This may require the root CA certificate to be included in the chain for full verification. The `openssl verify` command will indicate if any intermediate or root certificates are missing.

### Example: Complete Verification Workflow

```bash
# 1. Verify certificate matches private key
openssl x509 -noout -modulus -in cert.pem | openssl md5
openssl rsa -noout -modulus -in private.key | openssl md5

# 2. Check certificate validity dates
openssl x509 -in cert.pem -noout -dates

# 3. Verify SAN entries
openssl x509 -in cert.pem -text -noout | grep -A 2 "Subject Alternative Name"

# 4. Verify certificate chain (if chain.pem is available)
openssl verify -CAfile fullchain.pem cert.pem

# 5. View full certificate details
openssl x509 -in cert.pem -text -noout
```

## Common Issues

### Certificate and Key Don't Match

If the modulus hashes don't match:
- You may have used a different private key to generate the CSR
- The certificate may have been issued for a different key pair
- **Solution:** Regenerate the CSR with the correct private key or obtain a new certificate

### Certificate Validation Fails

If `openssl verify` fails:
- Missing intermediate certificates in the chain
- Root CA certificate not in system trust store
- Certificate expired or not yet valid
- **Solution:** Ensure you have the complete certificate chain including intermediates

### SAN Not Present in Certificate

If the SAN extension is missing from the issued certificate:
- The CA may not have included the SAN from your CSR
- Your CSR may not have included SAN extension properly
- **Solution:** Verify your CSR with `openssl req -text` before submission, and ensure your CA supports SAN extensions

## Security Best Practices

1. **Protect your private key:**
   - Store private keys with restrictive permissions: `chmod 600 private.key`
   - Never share private keys or commit them to version control
   - Use strong passphrases for encrypted keys (though note that `-nodes` flag creates unencrypted keys for automation)

2. **Verify before deploying:**
   - Always verify certificate matches private key before deployment
   - Check certificate expiration dates
   - Verify all required SAN entries are present

3. **Keep backups:**
   - Backup private keys securely (encrypted)
   - Keep a copy of CSRs for reference
   - Document which CA issued the certificate and when

## Related Documentation

- [Certificate Rotation Guidelines](./rotation.md)
- [Certificate Export Specifications](../openspec/specs/certificate-export/spec.md)
