# Lesson 7: Certificates and Trust (X.509)

## Real-life analogy: the passport

You arrive at a foreign border. How does the officer know you are who you claim to be?

```
┌──────────────────────────────────────────────────────────┐
│  You:     "I'm Alice from the US"                        │
│  Officer: "Prove it"                                     │
│  You:     (shows passport)                               │
│                                                          │
│  Passport:                         Certificate:          │
│    Name: Alice Smith                 Subject: example.com│
│    Photo: (your face)                Public key: 0x3a8f..│
│    Issued by: US Government          Issuer: Let's Encrypt│
│    Expires: 2030-01-01              Expires: 2025-01-01 │
│    Signature: (government stamp)     Signature: (CA's)   │
│                                                          │
│  Officer checks:                   Browser checks:       │
│    1. Is the issuer trusted?         1. Trusted CA?       │
│    2. Is the signature valid?        2. Valid signature?  │
│    3. Has it expired?                3. Expired?          │
│    4. Does the photo match?          4. Hostname match?   │
└──────────────────────────────────────────────────────────┘
```

A certificate IS a digital passport. It binds an identity (domain name) to a public key, signed by a trusted authority.

## The missing piece

You can now exchange keys (Lesson 4), derive encryption keys (Lesson 5), and encrypt data (Lesson 2). But there's a fatal flaw: **how does the client know it's talking to the real server?**

## The man-in-the-middle attack

Without authentication, an attacker (Mallory) sits between Alice and Bob:

```
Alice ←──DH──→ Mallory ←──DH──→ Bob
       key_1             key_2
```

- Alice thinks she did DH with Bob. She actually did DH with Mallory → `key_1`
- Bob thinks he did DH with Alice. He actually did DH with Mallory → `key_2`
- Mallory decrypts Alice's messages with `key_1`, reads them, re-encrypts with `key_2`, sends to Bob
- Neither Alice nor Bob detects anything wrong

All the encryption in the world doesn't help if you're encrypting to the wrong person.

## Certificates: binding identity to public keys

A certificate is a signed document that says:

```
┌──────────────────────────────────────┐
│ X.509 Certificate                    │
│                                      │
│ Subject:    server.example.com       │
│ Public Key: 0x3a8f7b...             │
│ Issuer:     Let's Encrypt            │
│ Valid:      2024-01-01 to 2025-01-01 │
│ Serial:     12345                    │
│                                      │
│ Signature:  0xab12... (signed by     │
│             issuer's private key)    │
└──────────────────────────────────────┘
```

The issuer (Certificate Authority) vouches: "I verified that the entity controlling `server.example.com` holds the private key corresponding to public key `0x3a8f7b...`."

## Chain of trust

Who vouches for the CA? Another CA, all the way up to a **Root CA**:

```
Root CA (pre-installed on your OS — Apple, Google, Mozilla maintain these lists)
  │
  └─ signs → Intermediate CA certificate (e.g., Let's Encrypt R3)
               │
               └─ signs → Server certificate (e.g., example.com)
```

Your browser/OS ships with ~150 trusted root CA certificates. When a server presents its certificate:

1. Read the server certificate → signed by Intermediate CA
2. Read the Intermediate CA certificate → signed by Root CA
3. Root CA is in the trusted store → **chain is valid**
4. Verify the server certificate's subject matches the hostname you're connecting to

If any link breaks — wrong signature, expired cert, hostname mismatch — the connection is rejected.

### See it yourself

```sh
# View a real website's certificate chain:
echo | openssl s_client -connect google.com:443 -showcerts 2>/dev/null | \
  grep -E "subject=|issuer="
# subject=CN = *.google.com
# issuer=CN = GTS CA 1C3           ← intermediate CA
# subject=CN = GTS CA 1C3
# issuer=CN = GTS Root R1          ← root CA

# See the full certificate details:
echo | openssl s_client -connect google.com:443 2>/dev/null | \
  openssl x509 -text -noout | head -30

# Check when it expires:
echo | openssl s_client -connect google.com:443 2>/dev/null | \
  openssl x509 -noout -dates
# notBefore=...
# notAfter=...

# See the Subject Alternative Names (what domains this cert covers):
echo | openssl s_client -connect google.com:443 2>/dev/null | \
  openssl x509 -noout -ext subjectAltName
# DNS:*.google.com, DNS:google.com, DNS:*.youtube.com, ...
```

```sh
# See which Root CAs your OS trusts:
# macOS:
security find-certificate -a /System/Library/Keychains/SystemRootCertificates.keychain | \
  grep "alis" | wc -l
# ~150 trusted root certificates

# Linux:
ls /etc/ssl/certs/ | wc -l
# Or:
awk -v cmd='openssl x509 -noout -subject' '/BEGIN/{close(cmd)};{print | cmd}' \
  /etc/ssl/certs/ca-certificates.crt 2>/dev/null | wc -l
```

```sh
# Generate a self-signed certificate:
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout server.key -out server.crt \
  -days 365 -subj "/CN=localhost"

# Inspect it:
openssl x509 -in server.crt -text -noout | head -20
# Note: Issuer == Subject (self-signed)
```

## Self-signed certificates

A self-signed certificate signs itself — it's both the subject and the issuer. No chain of trust; the client must explicitly trust it.

Used for:
- Development and testing
- Internal infrastructure (company VPNs, private services)
- Scenarios where you control both client and server

This is analogous to WireGuard: you manually exchange public keys rather than using a CA hierarchy.

## Real-world scenarios

### Alice visits her bank's website

1. Alice navigates to `https://bank.com`
2. Bank's server sends its certificate: "I am bank.com, here's my public key, signed by DigiCert"
3. Alice's browser checks:
   - Is DigiCert's certificate in the trusted root store? **Yes**
   - Does DigiCert's signature on bank.com's certificate verify? **Yes**
   - Is the certificate still valid (not expired)? **Yes**
   - Does the subject match the URL? `bank.com` == `bank.com` **Yes**
4. Browser proceeds with TLS handshake using the server's public key
5. The padlock icon appears

If Mallory tries to MITM this, she can't forge a certificate for `bank.com` — she doesn't have DigiCert's private key. She could present her own self-signed certificate, but the browser would show a scary warning.

### Bob deploys a private service

Bob runs an internal API at `api.internal.corp`. He doesn't want to (or can't) use a public CA.

1. Bob generates a self-signed CA certificate (his own private root)
2. Bob generates a server certificate for `api.internal.corp`, signs it with his CA
3. Bob installs his CA certificate on all client machines
4. Clients trust `api.internal.corp` because it chains to Bob's CA

This is common in corporate environments, Kubernetes clusters, and development setups.

### Certificate pinning (extra security)

Instead of trusting any CA to vouch for a server, the client hardcodes the expected certificate (or public key hash). Even if a CA is compromised, the attacker can't forge a pin-matching certificate.

Used by: banking apps, Signal, some browsers for critical services (Google pins its own certs in Chrome).

### The Let's Encrypt revolution

Before 2015, certificates cost money ($50-300/year) and required manual verification. Let's Encrypt automated the process:

1. You prove you control a domain (by placing a file on your web server or adding a DNS record)
2. Let's Encrypt issues a free certificate, valid for 90 days
3. Automated renewal via certbot

This made HTTPS the default for the entire web. Over 80% of web traffic is now encrypted, up from ~30% in 2014.

## Certificate formats

- **PEM**: Base64-encoded, delimited by `-----BEGIN CERTIFICATE-----`. Human-readable, used by most tools.
- **DER**: Raw binary encoding. Same data as PEM, just not base64-encoded.
- **PKCS#12 / PFX**: Bundles certificate + private key in one encrypted file. Common on Windows.

## Exercises

### Exercise 1: Parse a certificate

Generate a self-signed cert with openssl, read it in Rust using `rustls-pemfile` + `x509-parser`, print subject and public key algorithm.

```sh
# Generate:
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout server.key -out server.crt -days 365 -subj "/CN=localhost"

# Your Rust program should output:
# Subject: CN=localhost
# Public key: rsaEncryption
```

### Exercise 2: Certificate details

Extend the parser to print:
- Issuer name (should equal Subject for self-signed)
- Validity dates (not before / not after)
- Serial number
- Signature algorithm
- Subject Alternative Names (if any)

Compare your output with `openssl x509 -in server.crt -text -noout`.

### Exercise 3: Verify the self-signature

For a self-signed cert, the issuer's public key = subject's public key. Extract both and verify the signature. The `x509-parser` crate's `verify_signature` method can help.

### Exercise 4: Download and parse a real certificate chain

```sh
# Download google.com's chain:
echo | openssl s_client -connect google.com:443 -showcerts 2>/dev/null > chain.pem
```

Parse each certificate in the PEM file. Print subject/issuer for each. You should see the chain: `*.google.com → GTS CA 1C3 → GTS Root R1`.

### Exercise 5: Certificate expiry checker

Build a CLI tool that connects to a list of domains, downloads their certificates, and reports days until expiry:

```sh
cargo run -p tls --bin 7-certs -- check google.com github.com expired.badssl.com
# google.com:  expires in 62 days  ✓
# github.com:  expires in 198 days ✓
# expired.badssl.com: EXPIRED 847 days ago  ✗
```

The site `expired.badssl.com` has an intentionally expired cert — great for testing.
