# TLS Learning Plan

## Phase 1: Cryptographic Building Blocks

### Lesson 1: Hashing (SHA-256) ✅
- What a hash function is: fixed-size fingerprint of any data
- Properties: deterministic, one-way, avalanche effect, collision resistant
- TLS uses: integrity, HMAC, key derivation, certificate fingerprints, handshake transcript
- **Exercise**: `src/bin/1-hash.rs` — CLI tool that SHA-256 hashes a file, verified against `shasum -a 256`

### Lesson 2: Symmetric Encryption (ChaCha20-Poly1305) ✅
- One key encrypts and decrypts, fast (gigabytes/sec)
- AEAD: encryption + authentication in one operation (ciphertext + 16-byte tag)
- ChaCha20 = stream cipher, Poly1305 = MAC
- Nonces: 12 bytes, never reuse with the same key
- **Exercise**: `src/bin/2-encrypt.rs` — encrypt/decrypt a message, then tamper with ciphertext to show detection

### Lesson 3: Asymmetric Crypto & Signatures (Ed25519) ✅
- Key pairs: private key (secret) + public key (shared)
- Two uses: encryption (rare in TLS 1.3) and signatures (critical)
- TLS uses signatures to authenticate the server during handshake
- Ed25519: fast, 32-byte keys, 64-byte signatures
- **Exercise**: `src/bin/3-sign.rs` — generate key pair, sign message, verify, then tamper to show failure

### Lesson 4: Diffie-Hellman Key Exchange (X25519) ⬜
- Problem: how to agree on a shared secret over an insecure channel
- Each side generates ephemeral key pair, exchanges public keys, derives same shared secret
- Forward secrecy: new ephemeral keys each connection, past sessions can't be decrypted
- **Exercise**: `src/bin/4-keyexchange.rs` — Alice and Bob do X25519, print both shared secrets to prove they match

## Phase 2: Putting Primitives Together

### Lesson 5: HMAC and Key Derivation (HKDF) ⬜
- HMAC: hash + secret key = message authentication
- KDF: deriving multiple keys from one shared secret
- HKDF: extract (concentrate entropy) + expand (derive multiple keys)
- **Exercise**: `src/bin/5-kdf.rs` — take shared secret from Lesson 4, use HKDF to derive two 32-byte keys (client→server, server→client)

### Lesson 6: Certificates and Trust (X.509) ⬜
- Certificate: binds public key to identity, signed by CA
- Chain of trust: Root CA → Intermediate CA → End entity
- Self-signed vs CA-signed
- **Exercise**: `src/bin/6-certs.rs` — generate self-signed cert with openssl CLI, read it in Rust, extract public key and subject

## Phase 3: Build a Mini-TLS

### Lesson 7: Encrypted Echo Server (no auth) ⬜
- Combine: key exchange → key derivation → encrypted communication
- No certificates yet — just DH + symmetric encryption
- This is TLS without authentication
- **Exercise**: `src/bin/7-echo-server.rs` + `src/bin/7-echo-client.rs` — TCP echo pair with X25519 handshake + ChaCha20-Poly1305 encryption

### Lesson 8: Add Authentication ⬜
- Man-in-the-middle: without auth, attacker can intercept key exchange
- Solution: server signs its ephemeral public key with long-term Ed25519 key
- Client verifies signature against known public key
- **Exercise**: extend Lesson 7 — server signs ephemeral key, client verifies before proceeding

## Phase 4: Real TLS

### Lesson 9: Add TLS to the Tunnel (tokio-rustls) ⬜
- `rustls`: pure-Rust TLS library, `tokio-rustls`: async wrapper
- Generate certs with `rcgen` or openssl
- Wrap TcpStream with TlsStream in tunnel-client and tunnel-server
- **Exercise**: add TLS to the real tunnel, verify all existing tests still pass
