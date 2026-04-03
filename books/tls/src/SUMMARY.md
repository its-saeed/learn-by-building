# Summary

[Introduction](./introduction.md)

# Phase 1: Cryptographic Building Blocks

- [Cryptography Fundamentals](./00-fundamentals.md)
- [Hashing (SHA-256)](./01-hash.md)
- [Symmetric Encryption (ChaCha20-Poly1305)](./02-encrypt.md)
- [Asymmetric Crypto & Signatures (Ed25519)](./03-sign.md)
- [Key Exchange (X25519)](./04-keyexchange.md)

# Phase 2: Putting Primitives Together

- [Key Derivation (HKDF)](./05-kdf.md)
- [Certificates & Trust (X.509)](./06-certs.md)

# Phase 3: Build a Mini-TLS

- [Encrypted Echo Server](./07-echo-server.md)
- [Authenticated Echo Server](./08-echo-server.md)
- [Mutual TLS (mTLS)](./09-mtls.md)
- [Replay Attack Defense](./10-replay.md)

# Phase 4: Real TLS

- [Real TLS (tokio-rustls)](./11-real-tls.md)
- [HTTPS Client](./12-https-client.md)
