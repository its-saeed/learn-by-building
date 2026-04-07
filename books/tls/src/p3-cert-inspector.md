# Project: Certificate Inspector

> **Prerequisites**: Lesson 7 (Certificates), Lesson 14 (tokio-rustls). Connect to real websites and inspect their TLS certificates.

## What you're building

A CLI tool that connects to any website over TLS, downloads its certificate chain, and displays the details — like a mini `openssl s_client`.

```sh
cargo run -p tls --bin p3-cert-inspector -- google.com

  google.com:443
  ──────────────
  Certificate chain:
    [0] *.google.com
        Issuer:     GTS CA 1C3
        Valid:      2024-10-21 to 2025-01-13
        Key:        EC (P-256)
        SANs:       *.google.com, google.com, *.youtube.com, ...

    [1] GTS CA 1C3
        Issuer:     GTS Root R1
        Valid:      2020-08-13 to 2027-09-30
        Key:        RSA (2048 bits)

    [2] GTS Root R1 (trust anchor)
        Self-signed root CA

  Protocol:   TLS 1.3
  Cipher:     TLS_AES_256_GCM_SHA384
  Expires in: 42 days
```

## What you'll learn

This project teaches you to:
- Establish a TLS connection with `tokio-rustls`
- Extract certificates from the handshake
- Parse X.509 certificates with `x509-parser`
- Display the certificate chain and verify trust

## The reference tool

```sh
# This is what your tool replaces:
echo | openssl s_client -connect google.com:443 -showcerts 2>/dev/null | \
  openssl x509 -text -noout | head -30

# Certificate chain:
echo | openssl s_client -connect google.com:443 2>/dev/null | \
  grep -E "subject=|issuer=|depth="

# Expiry:
echo | openssl s_client -connect google.com:443 2>/dev/null | \
  openssl x509 -noout -dates

# SANs:
echo | openssl s_client -connect google.com:443 2>/dev/null | \
  openssl x509 -noout -ext subjectAltName
```

## Implementation guide

### Step 1: TLS connection

```rust
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use rustls::{ClientConfig, RootCertStore};

async fn connect(host: &str) -> Result</* TlsStream */> {
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let tcp = TcpStream::connect(format!("{host}:443")).await?;
    let server_name = host.try_into()?;
    let tls = connector.connect(server_name, tcp).await?;
    Ok(tls)
}
```

### Step 2: Extract certificates

```rust
let (_, conn) = tls_stream.get_ref();
let certs = conn.peer_certificates().unwrap();
// certs is a Vec<CertificateDer> — DER-encoded X.509 certificates
```

### Step 3: Parse with x509-parser

```rust
use x509_parser::prelude::*;

for (i, cert_der) in certs.iter().enumerate() {
    let (_, cert) = X509Certificate::from_der(cert_der)?;
    println!("[{i}] {}", cert.subject());
    println!("    Issuer: {}", cert.issuer());
    println!("    Valid:  {} to {}", cert.validity().not_before, cert.validity().not_after);
}
```

### Step 4: Extract SANs

```rust
if let Some(san) = cert.subject_alternative_name()? {
    for name in &san.value.general_names {
        match name {
            GeneralName::DNSName(dns) => println!("    SAN: {dns}"),
            GeneralName::IPAddress(ip) => println!("    SAN: {ip:?}"),
            _ => {}
        }
    }
}
```

## Test targets

```sh
# Normal sites:
cargo run -p tls --bin p3-cert-inspector -- google.com github.com cloudflare.com

# Interesting cases:
cargo run -p tls --bin p3-cert-inspector -- expired.badssl.com    # expired cert
cargo run -p tls --bin p3-cert-inspector -- wrong.host.badssl.com # hostname mismatch
cargo run -p tls --bin p3-cert-inspector -- self-signed.badssl.com # self-signed

# badssl.com provides test endpoints for every TLS edge case
```

## Exercises

### Exercise 1: Basic inspector

Connect, extract chain, print subject/issuer/validity for each cert. Compare output with `openssl s_client`.

### Exercise 2: Expiry checker

Check multiple domains and report days until expiry:
```sh
cargo run -p tls --bin p3-cert-inspector -- --check-expiry google.com github.com
# google.com:  62 days remaining ✓
# github.com: 198 days remaining ✓
```

### Exercise 3: Certificate pinning check

Download a site's certificate, compute its SHA-256 fingerprint. Compare with a known pin. Report if it matches or changed since last check.

```sh
cargo run -p tls --bin p3-cert-inspector -- --pin google.com
# Fingerprint: SHA-256:a1b2c3d4...
# Save to pins.json, check again later
```

### Exercise 4: JSON output

Add `--json` flag for machine-readable output:
```json
{
  "host": "google.com",
  "chain": [
    { "subject": "*.google.com", "issuer": "GTS CA 1C3", "expires": "2025-01-13" }
  ],
  "protocol": "TLS 1.3",
  "cipher": "TLS_AES_256_GCM_SHA384"
}
```
