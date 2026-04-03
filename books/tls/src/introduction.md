# TLS from Scratch

Build TLS from its cryptographic primitives. Each lesson introduces one concept with theory, real-world scenarios, and hands-on Rust exercises.

You'll go from hashing a file to building an encrypted, authenticated communication channel — understanding every layer.

## What you'll build

- SHA-256 file hasher
- ChaCha20-Poly1305 encryption/decryption
- Ed25519 digital signatures
- X25519 Diffie-Hellman key exchange
- HKDF key derivation
- X.509 certificate parser
- Encrypted echo server (your own mini-TLS)
- Authenticated echo server with Ed25519 signatures
- Mutual TLS (mTLS)
- Replay attack defense with counter nonces

## Prerequisites

- Rust fundamentals
- Basic networking (TCP/UDP)

## Source code

```sh
git clone https://github.com/its-saeed/learn-by-building.git
cd learn-by-building
cargo run -p tls --bin 1-hash -- --file-path Cargo.toml
```
