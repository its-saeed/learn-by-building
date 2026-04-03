# Lesson 0: Cryptography Fundamentals

Before writing any code, you need to understand the vocabulary and core concepts. Every lesson that follows builds on these ideas.

## The three goals of cryptography

### 1. Confidentiality

Only the intended recipient can read the message. Everyone else sees random noise.

```
Alice writes: "meet at 3pm"
Alice encrypts: "meet at 3pm" → "x7#kQ!9pL@"
Eve intercepts: "x7#kQ!9pL@" (meaningless)
Bob decrypts: "x7#kQ!9pL@" → "meet at 3pm"
```

**Without confidentiality**: anyone on the network (ISP, router, attacker on the same Wi-Fi) can read your emails, passwords, bank transfers, medical records.

### 2. Integrity

The message hasn't been modified in transit. If anyone changes even one bit, the recipient detects it.

```
Alice sends: "transfer $100 to Bob"
Eve intercepts, modifies: "transfer $999 to Eve"
Bob receives it — but integrity check FAILS → message rejected
```

**Without integrity**: an attacker can silently alter messages. Change a dollar amount, modify a software update to include malware, alter DNS responses to redirect traffic.

### 3. Authentication

You know who you're talking to. The sender is who they claim to be.

```
Alice receives a message claiming to be from her bank.
Authentication check: is this really from the bank, or from an attacker pretending to be the bank?
```

**Without authentication**: phishing, man-in-the-middle attacks, impersonation. An attacker sets up a fake bank website — without authentication, your browser can't tell the difference.

## Core terminology

### Plaintext and ciphertext

- **Plaintext**: the original, readable data. Doesn't have to be text — could be a file, image, or any bytes.
- **Ciphertext**: the encrypted, unreadable version. Looks like random bytes. Same length as plaintext (roughly).
- **Encryption**: plaintext → ciphertext (using a key)
- **Decryption**: ciphertext → plaintext (using a key)

```
plaintext: "hello world"
     │
     ▼ encrypt(key)
ciphertext: 0x7a3f8b2e1c...
     │
     ▼ decrypt(key)
plaintext: "hello world"
```

### Keys

A key is a secret value that controls encryption and decryption. Without the key, decryption is computationally impossible.

- **Symmetric key**: one key for both encryption and decryption. Both sides must share the same key.
- **Asymmetric key pair**: two keys — a public key (shared openly) and a private key (kept secret). What one encrypts, only the other can decrypt.

```
Symmetric:
  Alice and Bob both have key K
  Alice: encrypt(K, plaintext) → ciphertext
  Bob:   decrypt(K, ciphertext) → plaintext

Asymmetric:
  Bob has: public key (shared) + private key (secret)
  Alice: encrypt(Bob_public, plaintext) → ciphertext
  Bob:   decrypt(Bob_private, ciphertext) → plaintext
```

### Cipher

An algorithm that performs encryption and decryption. Examples:
- **AES** (Advanced Encryption Standard): symmetric, block cipher
- **ChaCha20**: symmetric, stream cipher
- **RSA**: asymmetric

The cipher is public — security comes from the key, not from keeping the algorithm secret. This is **Kerckhoffs's principle**: a system should be secure even if everything about it is public knowledge except the key.

### Hash / Digest

A fixed-size fingerprint of any data. One-way — you can't reverse it.

```
SHA-256("hello") → 2cf24dba5fb0a30e...  (always 32 bytes)
SHA-256("hello ") → 98ea6e4f216f2fb4... (completely different)
```

Used for: integrity verification, password storage, key derivation, digital signatures.

### Nonce

"Number used once." A value that must never repeat with the same key. Used in encryption to ensure that encrypting the same plaintext twice produces different ciphertext.

```
encrypt(key, nonce=1, "hello") → 0x8a3f...
encrypt(key, nonce=2, "hello") → 0x2b7c...  (different!)
```

If you reuse a nonce with the same key, the encryption breaks — an attacker can recover plaintext. This is one of the most common crypto implementation mistakes.

### Digital signature

The asymmetric equivalent of a handwritten signature. Proves who created a message and that it hasn't been modified.

```
Alice signs:   signature = sign(Alice_private_key, message)
Bob verifies:  verify(Alice_public_key, message, signature) → true/false
```

If the message changes, verification fails. If someone else tries to sign, they can't produce a valid signature without Alice's private key.

### MAC (Message Authentication Code)

Like a digital signature, but using a symmetric key. Both sides need the same secret key to create and verify the MAC.

```
tag = MAC(shared_key, message)
```

Anyone with the shared key can verify the tag. Unlike a signature, a MAC doesn't prove *which* key holder created it (both sides have the same key).

**HMAC**: MAC built from a hash function (e.g., HMAC-SHA256). Used extensively in TLS.

### AEAD (Authenticated Encryption with Associated Data)

Modern encryption that provides both confidentiality AND integrity in one operation. You get:
- **Ciphertext**: encrypted data (confidentiality)
- **Authentication tag**: proves the ciphertext wasn't modified (integrity)

"Associated data" is metadata that's authenticated but not encrypted (e.g., a message header).

```
encrypt(key, nonce, plaintext, associated_data) → (ciphertext, tag)
decrypt(key, nonce, ciphertext, tag, associated_data) → plaintext OR error
```

If anyone modifies the ciphertext, the tag, or the associated data, decryption fails.

## Authentication vs Authorization

These are different concepts that are often confused:

- **Authentication** (AuthN): "Who are you?" — verifying identity
  - Example: logging in with username/password, presenting a certificate
- **Authorization** (AuthZ): "What are you allowed to do?" — verifying permissions
  - Example: "user X can read this file but not write to it"

TLS handles **authentication** (proving the server is who it claims to be). It does NOT handle authorization — that's the application's job.

```
TLS handshake: "I am server.example.com" (authentication)
Application:   "User alice can access /admin" (authorization)
```

## Trust models

How do you decide to trust a public key?

### Direct trust (pinned keys)
You manually verify and store the public key. Simple but doesn't scale.
- Example: SSH `known_hosts`, WireGuard peer configuration

### Web of trust
People vouch for each other's keys. Decentralized but messy.
- Example: PGP/GPG key signing parties

### Certificate authority (CA)
A trusted third party vouches for public keys by signing certificates. Hierarchical and scalable.
- Example: HTTPS (Let's Encrypt, DigiCert sign server certificates)

### Trust on first use (TOFU)
Accept the key the first time you see it, alert if it changes.
- Example: SSH ("The authenticity of host can't be established... continue?")

## Forward secrecy

If an attacker records all your encrypted traffic today, and steals your long-term key next year, can they decrypt the recorded traffic?

- **Without forward secrecy**: Yes. The long-term key decrypts everything.
- **With forward secrecy**: No. Each session used ephemeral keys that were destroyed. The long-term key can't help.

TLS 1.3 mandates forward secrecy by requiring ephemeral Diffie-Hellman key exchange.

## The cast of characters

Cryptography literature uses standard names:

| Name | Role |
|------|------|
| **Alice** | Initiator (usually the client) |
| **Bob** | Responder (usually the server) |
| **Eve** | Eavesdropper — passively listens to traffic |
| **Mallory** | Active attacker — can modify, inject, and replay messages |
| **Trent** | Trusted third party (e.g., a Certificate Authority) |

## Common attacks

### Eavesdropping (passive)
Eve listens to network traffic. Defeated by encryption (confidentiality).

### Man-in-the-middle / MITM (active)
Mallory sits between Alice and Bob, impersonating each to the other. Defeated by authentication.

```
Alice ←→ Mallory ←→ Bob
Alice thinks she's talking to Bob.
Bob thinks he's talking to Alice.
Mallory reads and modifies everything.
```

### Replay attack
Mallory records a valid encrypted message and sends it again later. Defeated by sequence numbers or timestamps.

### Tampering
Mallory modifies an encrypted message in transit. Defeated by integrity checks (AEAD, MAC).

### Downgrade attack
Mallory forces Alice and Bob to use weaker crypto than they'd normally choose. Defeated by signing the handshake negotiation.

## How TLS uses all of this

```
TLS Handshake:
  1. Negotiate cipher suite          (which algorithms to use)
  2. Key exchange (DH)               (confidentiality + forward secrecy)
  3. Server certificate              (authentication via CA trust model)
  4. Server signature                (proves server has the private key)
  5. Key derivation (HKDF)           (derive session keys)
  6. Finished messages               (integrity of handshake — MAC)

TLS Record Protocol:
  7. AEAD encryption of data         (confidentiality + integrity)
  8. Sequence number nonces           (replay defense)
```

Every concept in this lesson maps to a specific part of TLS. The following lessons implement each piece in Rust.
