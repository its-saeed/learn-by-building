# TLS from Scratch

Build TLS from its cryptographic primitives. Each lesson introduces one concept with real-life analogies, shell commands, ASCII diagrams, and hands-on Rust exercises.

You'll go from hashing a file to building an encrypted, authenticated communication channel — then use real TLS libraries and build production tools.

## Course overview

```
Phase 1: Cryptographic Building Blocks
  Hashing → Encryption → Signatures → Key Exchange
  + Projects: TOTP Authenticator, Signed Git Commits

Phase 2: Putting Primitives Together
  Key Derivation → Password KDFs → Certificates → Cert Generation
  + Projects: Certificate Inspector, Password Manager Vault

Phase 3: Build a Mini-TLS
  Encrypted Echo → Authenticated Echo → mTLS → Replay Defense → Handshake Deep Dive
  + Project: Encrypted File Transfer

Phase 4: Real TLS
  tokio-rustls → HTTPS Client
  + Projects: HTTPS Server, TLS Scanner

Phase 5: Capstone Projects
  Certificate Authority, mTLS Service Mesh, TLS Proxy, Intercepting Proxy
```

## What you'll build

**Lessons** (16 total):
- SHA-256 file hasher
- ChaCha20-Poly1305 encryption with tamper detection
- Ed25519 digital signatures
- X25519 Diffie-Hellman key exchange
- HKDF key derivation + password-based KDFs (Argon2)
- X.509 certificate parsing + generation with `rcgen`
- Encrypted echo server (your own mini-TLS)
- Authenticated echo server, mutual TLS, replay defense
- TLS 1.3 handshake deep dive
- Real TLS with `tokio-rustls`, HTTPS client

**Projects** (11 total):
- TOTP authenticator (Google Authenticator clone)
- Signed git commits
- Certificate inspector (check any website's cert chain)
- Password manager vault
- Encrypted file transfer (mini `scp`)
- HTTPS server
- TLS scanner (probe server configurations)
- Certificate authority
- mTLS service mesh
- TLS termination proxy
- HTTPS intercepting proxy (mini mitmproxy)

## Prerequisites

- Rust fundamentals (ownership, traits, generics)
- Basic networking (TCP/UDP)

## Source code

```sh
git clone https://github.com/its-saeed/learn-by-building.git
cd learn-by-building
cargo run -p tls --bin 1-hash -- --file-path Cargo.toml
```
