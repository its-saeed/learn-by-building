# Lesson 2: Symmetric Encryption (ChaCha20-Poly1305)

## The idea

You have one secret key. The same key encrypts and decrypts. Both sides must know the key beforehand.

```
plaintext + key → ciphertext
ciphertext + key → plaintext
```

This is what TLS uses for all bulk data after the handshake. It's fast — gigabytes per second on modern hardware.

## AEAD: Authenticated Encryption with Associated Data

Old encryption (like AES-CBC) only gave you **confidentiality** — an attacker couldn't read your data, but could silently **flip bits** in the ciphertext. You'd decrypt garbage without knowing it was tampered with. This led to real attacks (padding oracle, etc.).

AEAD solves this by combining encryption + integrity in one operation:
- **Ciphertext**: same length as plaintext (encrypted)
- **Tag**: 16 bytes appended — a MAC that proves the ciphertext wasn't modified

When decrypting, if even one bit of the ciphertext or tag was changed, decryption **fails with an error** instead of silently returning corrupted data.

## ChaCha20-Poly1305

TLS 1.3 supports exactly two AEAD ciphers:
- **AES-256-GCM**: hardware-accelerated on most CPUs via AES-NI instructions
- **ChaCha20-Poly1305**: faster in pure software (no special CPU instructions needed), designed by Daniel Bernstein

ChaCha20 = the encryption part (stream cipher — generates a keystream XORed with plaintext)
Poly1305 = the authentication part (computes a MAC over ciphertext)

## Nonces: the critical rule

Every encryption call takes a **nonce** (number used once) — 12 bytes. The absolute rule:

**Never reuse a nonce with the same key.**

If you encrypt two different messages with the same key and nonce, an attacker can XOR the two ciphertexts together — the keystreams cancel out, revealing the XOR of the two plaintexts. From there, frequency analysis recovers both messages.

In TLS, the nonce is simply a counter (0, 1, 2, ...) — trivial to guarantee uniqueness.

## Real-world scenarios

### Alice and Bob's encrypted chat

Alice and Bob pre-share a 256-bit key (how they share it is Lesson 4's problem).

1. Alice wants to send "meet at 3pm"
2. Alice encrypts with key + nonce=0: `encrypt(key, 0, "meet at 3pm")` → ciphertext + tag
3. Alice sends ciphertext + tag to Bob
4. Bob decrypts with the same key + nonce=0 → "meet at 3pm"

Eve intercepts the ciphertext. She sees random-looking bytes. She flips a byte hoping to change "3pm" to "5pm". Bob tries to decrypt — the tag check fails. Bob knows the message was tampered with.

### The nonce disaster: reuse in practice

In 2016, researchers found that a popular TLS implementation reused nonces when session tickets were used across multiple servers. This allowed attackers to recover plaintext from recorded sessions. A simple counter bug broke the entire encryption.

### Why two different ciphers in TLS?

AES-GCM is faster on CPUs with AES-NI (Intel, AMD, modern ARM). But on devices without hardware AES support (older phones, IoT), AES-GCM is slow. ChaCha20-Poly1305 is consistently fast everywhere because it only uses basic operations (add, rotate, XOR). Google originally pushed for ChaCha20-Poly1305 in TLS specifically for Android devices.

## Exercises

### Exercise 1: Encrypt, decrypt, tamper (implemented in 2-encrypt.rs)
Encrypt a message, decrypt it, then flip one byte and show decryption fails.

### Exercise 2: Nonce reuse attack (try it yourself)
Encrypt two different messages with the SAME key and SAME nonce. XOR the two ciphertexts together. Compare with XORing the two plaintexts together. They should be identical — demonstrating why nonce reuse is catastrophic.

```rust
let c1 = cipher.encrypt(nonce, b"hello world!".as_ref()).unwrap();
let c2 = cipher.encrypt(nonce, b"secret msg!!".as_ref()).unwrap();
// XOR c1 and c2 (skip the 16-byte tag at the end)
// Compare with XOR of "hello world!" and "secret msg!!"
```

### Exercise 3: Large file encryption
Encrypt a 1MB file in chunks. Use a counter nonce: chunk 0 gets nonce=0, chunk 1 gets nonce=1, etc. Decrypt all chunks and verify the output matches the original file. This is how TLS encrypts a stream of data — one record at a time, each with an incrementing nonce.

### Exercise 4: Associated data
AEAD supports "associated data" — unencrypted metadata that's still authenticated. Try encrypting with AAD:
```rust
use chacha20poly1305::aead::Payload;
let payload = Payload { msg: b"secret body", aad: b"message-id: 42" };
cipher.encrypt(nonce, payload)
```
The AAD isn't encrypted (it's sent in plaintext alongside the ciphertext), but if anyone modifies it, decryption fails. TLS uses this for record headers — the header is plaintext but authenticated.
