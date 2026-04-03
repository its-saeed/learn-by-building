# Lesson 3: Asymmetric Crypto & Signatures (Ed25519)

## The problem symmetric crypto can't solve

In Lesson 2, both sides need the same key. But how do you share that key in the first place? You can't send it over the network — anyone watching would see it. You can't encrypt it — you'd need another key for that (chicken-and-egg).

Asymmetric crypto solves this with **key pairs**:
- **Private key**: kept secret, never leaves your machine
- **Public key**: given to everyone

## Two uses of key pairs

### 1. Encryption (less common in modern TLS)
- Encrypt with someone's public key → only their private key can decrypt
- Used in older TLS (RSA key exchange), but NOT in TLS 1.3

### 2. Digital signatures (critical in TLS)
- Sign with your private key → anyone with your public key can verify
- Proves two things:
  - **Authenticity**: "this message was created by the private key holder"
  - **Integrity**: "this message hasn't been modified since signing"

## Ed25519

A modern signature algorithm based on elliptic curves (Curve25519). Designed by Daniel Bernstein.

- 32-byte private key, 32-byte public key, 64-byte signatures
- Very fast: ~15,000 signatures/second on a laptop
- Used by: SSH, WireGuard, Signal, many TLS implementations
- Deterministic: same key + same message always produces the same signature (no random nonce needed, unlike ECDSA)

```
sign(private_key, message) → signature (64 bytes)
verify(public_key, message, signature) → true/false
```

## Real-world scenarios

### Alice signs a software release

Alice publishes open-source software. Users need to verify downloads are genuinely from Alice, not an attacker who compromised the download mirror.

1. Alice generates an Ed25519 key pair. Publishes her public key on her website.
2. Alice builds version 2.0, signs the binary: `sign(alice_private, binary) → sig`
3. Alice uploads `binary` and `sig` to the download mirror
4. Bob downloads both. He has Alice's public key from her website.
5. Bob runs `verify(alice_public, binary, sig)` → success
6. An attacker modifies the binary on the mirror. Bob downloads it.
7. Bob runs `verify(alice_public, modified_binary, sig)` → **FAILS**

Bob knows the binary was tampered with. This is exactly how `apt` (Debian/Ubuntu) and `cargo` verify packages.

### Bob authenticates to a server (SSH)

When you run `ssh server.com`, the server proves its identity:

1. Server has a long-term Ed25519 key pair (in `/etc/ssh/ssh_host_ed25519_key`)
2. During SSH handshake, server signs session data with its private key
3. Client verifies the signature against the server's known public key (in `~/.ssh/known_hosts`)
4. If verification fails → "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!"

This prevents MITM attacks: an attacker can't forge the server's signature without its private key.

### How TLS uses signatures

During the TLS handshake:
1. Server sends its ephemeral DH public key (for key exchange, Lesson 4)
2. Server signs the handshake transcript (all messages so far) with its long-term private key
3. Client verifies the signature using the server's public key from the certificate (Lesson 6)
4. If the signature is valid → the client knows the DH public key genuinely came from the server
5. An attacker can't forge this because they don't have the server's private key

Without this signature, an attacker could substitute their own DH public key (man-in-the-middle attack).

### Ed25519 vs RSA vs ECDSA

| Algorithm | Key size | Signature size | Speed |
|-----------|----------|---------------|-------|
| RSA-2048 | 256 bytes | 256 bytes | Slow |
| ECDSA P-256 | 64 bytes | 64 bytes | Medium |
| Ed25519 | 32 bytes | 64 bytes | Fast |

RSA is the oldest, still widely used but being phased out. ECDSA has a dangerous footgun: if the random nonce leaks or is reused, the private key can be recovered (this happened to Sony's PS3 signing key in 2010). Ed25519 is deterministic — no random nonce, no footgun.

## Exercises

### Exercise 1: Sign and verify (implemented in 3-sign.rs)
Generate a key pair, sign a message, verify it. Then modify the message and show verification fails.

### Exercise 2: Sign multiple messages
Sign three different messages with the same key. Verify each with the corresponding message. Then try verifying message 1's signature against message 2 — it should fail. Each signature is bound to its specific message.

### Exercise 3: Key separation
Generate two different key pairs. Sign the same message with both. Show that:
- Key A's signature verifies with Key A's public key
- Key A's signature does NOT verify with Key B's public key
- Key B's signature does NOT verify with Key A's public key

This demonstrates that signatures are bound to both the message AND the signer's identity.

### Exercise 4: Detached signatures (real-world pattern)
Simulate a software release workflow:
1. Create a "binary" (any byte array)
2. Sign it, save the signature to a separate "file" (Vec<u8>)
3. In a separate function (simulating a different machine), load the "binary" and "signature", verify against a hardcoded public key
