# Lesson 0: Cryptography Fundamentals

> **The Story of Alice's Bookstore**
>
> Alice runs a small online bookstore from her apartment. She sells rare books and ships worldwide. Her customers type their credit card numbers into her website, send her messages, and download digital books.
>
> One day, her friend Bob — a security researcher — sits down with her at a coffee shop.
>
> *"Alice, your website sends everything in plaintext. Anyone on this Wi-Fi can see your customers' credit cards."*
>
> *"What? How?"*
>
> *"Let me show you the problems — and how cryptography solves each one."*
>
> This is where our story begins. Each lesson solves one of Alice's problems. By the end, her bookstore will be fully secure.

Before writing any code, you need to understand the vocabulary and core concepts. Every lesson that follows builds on these ideas.

## Real-life analogy: sending a secret letter

Imagine you need to send a secret letter across town. Every cryptographic concept maps to a part of this scenario:

You (Alice) want to send a letter to Bob. The mail carrier (the network) might read it.

| Cryptographic concept | Secret-letter analogy |
|-----------------------|-----------------------|
| Confidentiality | Put the letter in a locked box |
| Integrity | Seal the box with tamper-evident tape |
| Authentication | Stamp it with your wax seal |
| Key | The key to the locked box |
| Hash | A fingerprint of the letter |
| Signature | Your wax seal, made with a ring only you have |
| Nonce | A unique serial number on each box |
| Certificate | A passport proving you are Alice |

Without these protections, anyone can read the letter, change it, or pretend to be you.

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

## What cryptography does not solve

Cryptography is powerful, but it is not a magic shield around the whole application:

- It does not fix broken application logic. TLS can protect `POST /transfer`, but it cannot decide whether the transfer should be allowed.
- It does not handle authorization. TLS can prove "this is Alice" or "this is server.example.com"; your app still decides what that identity can do.
- It does not protect data after decryption. If the server logs plaintext credit card numbers, TLS cannot help.
- It does not save you if private keys are stolen. Attackers with the key can impersonate you until the key is rotated or revoked.
- It does not hide all metadata. Observers can still see IP addresses, ports, timing, traffic size, and usually the SNI hostname.

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

A key is a value that controls a cryptographic operation. Without the right key, decrypting data or producing valid signatures/MACs should be computationally impossible.

- **Symmetric key**: one key for both encryption and decryption. Both sides must share the same key.
- **Asymmetric key pair**: two keys — a public key (shared openly) and a private key (kept secret). Depending on the algorithm, key pairs are used for signing/verification, encryption/decryption, or key agreement.

```
Symmetric:
  Alice and Bob both have key K
  Alice: encrypt(K, plaintext) → ciphertext
  Bob:   decrypt(K, ciphertext) → plaintext

Asymmetric:
  Bob has: public key (shared) + private key (secret)
  Signature use: Bob signs with Bob_private, Alice verifies with Bob_public
  Encryption use: Alice encrypts to Bob_public, Bob decrypts with Bob_private
  Key exchange use: Alice and Bob combine key pairs to derive a shared secret
```

TLS 1.3 uses asymmetric cryptography mainly for **signatures** and **key exchange**, then uses fast symmetric encryption for the actual data.

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
SHA-256("hello ") → 5e3235a8346e5a45... (completely different)
```

Used for: integrity verification, password hashing/KDF schemes, key derivation, digital signatures.

### Encoding vs hashing vs encryption vs signing

These are easy to confuse:

| Operation | Secret key? | Reversible? | Main purpose |
|-----------|-------------|-------------|--------------|
| Encoding, e.g. Base64 | No | Yes | Make bytes easier to store or transmit |
| Hashing, e.g. SHA-256 | No | No | Fingerprint data |
| MAC, e.g. HMAC-SHA256 | Yes, shared key | No | Prove integrity and shared-key authenticity |
| Encryption, e.g. ChaCha20-Poly1305 | Yes | Yes, with key | Hide data |
| Signing, e.g. Ed25519 | Yes, private key | No | Prove who approved data |

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

A MAC is a short piece of data (a **tag**) that proves two things about a message:
1. **Integrity** — the message wasn't modified
2. **Authenticity** — the message came from someone who knows the secret key

Think of it as a tamper-evident seal that only works with a secret:

```
Creating a MAC:
  Alice has: message + shared_key
  Alice computes: tag = MAC(shared_key, "transfer $100")
  Alice sends: message + tag

Verifying a MAC:
  Bob has: message + tag + shared_key
  Bob computes: expected_tag = MAC(shared_key, "transfer $100")
  Bob checks: tag == expected_tag?
    YES → message is authentic and unmodified
    NO  → message was tampered with or wrong key

Attacker (no key):
  Eve intercepts: message + tag
  Eve changes message to "transfer $900"
  Eve can't compute new tag (doesn't have the key)
  Bob checks → tag doesn't match → REJECTED
```

**MAC vs Hash**: a plain hash (Lesson 1) proves integrity but NOT authenticity — anyone can compute `SHA-256("transfer $900")` and replace the hash. A MAC requires the secret key, so only key holders can produce valid tags.

**MAC vs Signature**: a MAC uses a **symmetric** key (both sides share the same key). A signature uses an **asymmetric** key pair. MAC is faster but doesn't prove *which* key holder created it (both sides have the same key). Signatures prove exactly who signed.

| Property | MAC | Signature |
|----------|-----|-----------|
| Key | Shared symmetric key | Asymmetric key pair |
| Creates | Anyone with the key | Only the private key holder |
| Verifies | Anyone with the key | Anyone with the public key |
| Proves | "A key holder made it" | "This specific private key holder signed it" |
| Speed | Fast | Slower |
| Used in TLS | Record integrity | Handshake authentication |

**HMAC**: the most common MAC construction — built from a hash function (e.g., HMAC-SHA256). "Hash-based MAC." Used extensively in TLS for key derivation (HKDF, Lesson 5) and handshake integrity.

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

TLS handles **authentication**. In normal HTTPS, the server proves it is who it claims to be. With mutual TLS (mTLS), the client also presents a certificate and proves its identity. TLS does NOT handle authorization — that's the application's job.

| Layer | Question answered |
|-------|-------------------|
| TLS handshake | "Is this really `server.example.com`?" |
| mTLS | "Is this really client certificate `alice`?" |
| Application | "Can Alice access `/admin`?" |

## Trust models

How do you decide to trust a public key? TLS needs an answer because a public key by itself is just bytes. The important question is: **does this key really belong to `example.com`?**

For this book, you only need the map:

| Model | Idea | Example |
|-------|------|---------|
| Direct trust / pinning | Store the exact expected key | SSH, WireGuard, certificate pinning |
| TOFU | Trust the first key, warn on change | SSH `known_hosts` |
| Certificate authority | A trusted third party signs a cert | HTTPS |

HTTPS uses the **certificate authority** model. The server sends a certificate containing its public key. A CA signs that certificate to say "this key belongs to this domain." Your browser already trusts a set of root CAs, so it can verify the chain.

Direct trust and TOFU are useful contrasts, but they do not scale to the public web. Certificates get their own lessons later.

## Forward secrecy

If an attacker records all your encrypted traffic today, and steals your long-term key next year, can they decrypt the recorded traffic?

- **Without forward secrecy**: Yes. The long-term key decrypts everything.
- **With forward secrecy**: No. Each session used ephemeral keys that were destroyed. The long-term key can't help.

In modern TLS, the long-term certificate key authenticates the server; it does **not** directly encrypt the session data. The actual traffic keys come from an ephemeral Diffie-Hellman exchange, then get destroyed when the session ends.

TLS 1.3 mandates this by requiring ephemeral Diffie-Hellman key exchange.

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

These attacks explain why TLS needs several building blocks. This is only a threat map; later lessons implement the defenses.

| Attack | Who | What they do | Defense |
|--------|-----|--------------|---------|
| Eavesdropping | Eve | Listens to traffic | Encryption |
| Man-in-the-middle | Mallory | Intercepts and modifies | Authentication |
| Replay | Mallory | Re-sends old messages | Nonces / sequence numbers |
| Tampering | Mallory | Modifies ciphertext | AEAD / MAC |
| Downgrade | Mallory | Forces weak crypto | Signed handshake |

- **Eavesdropping** is passive. Eve only listens. Encryption gives confidentiality.
- **Man-in-the-middle** is active. Mallory impersonates each side to the other. Certificates and signatures provide authentication.
- **Replay** means Mallory resends old valid bytes. Nonces, sequence numbers, and protocol state reject duplicates.
- **Tampering** means Mallory modifies data in transit. AEAD or MACs detect changes.
- **Downgrade** means Mallory tries to force weaker algorithms. TLS protects the negotiated parameters with the handshake transcript.

## See it in the real world

Every concept in this lesson is happening right now on your machine:

```sh
# See a real TLS handshake — every concept in action:
echo | openssl s_client -connect example.com:443 -servername example.com 2>/dev/null | head -20
# You'll see: certificate chain (authentication), cipher suite (encryption),
# protocol version, key exchange algorithm

# See WHICH cipher suite was negotiated:
echo | openssl s_client -connect example.com:443 -servername example.com 2>/dev/null | grep "Cipher"
# Example: TLS_AES_256_GCM_SHA384
# That's: AEAD cipher (AES-GCM) + hash (SHA384)

# See the certificate (authentication):
echo | openssl s_client -connect example.com:443 -servername example.com 2>/dev/null | \
  openssl x509 -noout -subject -issuer
# subject: ... example.com ...  ← who they claim to be
# issuer: ...                   ← who vouches for them (CA)

# See forward secrecy in action:
echo | openssl s_client -connect example.com:443 -servername example.com 2>/dev/null | grep -E "Server Temp Key|X25519|ECDH"
# Output depends on the server, but a TLS 1.3 connection uses ephemeral DH/ECDH keys.
```

```sh
# See HMAC (shared-key integrity):
printf 'transfer $100' | openssl dgst -sha256 -hmac "shared-secret"
# Change the message or the key and the tag changes completely.

# See a plain hash for comparison:
printf 'transfer $100' | shasum -a 256
# Anyone can compute this hash. Only someone with the shared secret can compute the HMAC.
```

Optional packet-capture experiment: capture a request to `https://example.com` with `tcpdump` or Wireshark. You will see IP addresses, ports, timing, and packet sizes, but the HTTP request and response body will be encrypted.

## How TLS uses all of this

| TLS part | Step | Purpose |
|----------|------|---------|
| Handshake | Negotiate cipher suite | Decide which algorithms to use |
| Handshake | Key exchange (DH) | Confidentiality + forward secrecy |
| Handshake | Server certificate | Authentication via the CA trust model |
| Handshake | Server signature | Prove the server has the private key |
| Handshake | Key derivation (HKDF) | Derive session keys |
| Handshake | Finished messages | Integrity of the handshake |
| Record protocol | AEAD encryption of data | Confidentiality + integrity |
| Record protocol | Sequence number nonces | Replay defense |

Every concept maps to a lesson:

| Concept | Lesson | What you'll build |
|---------|--------|-------------------|
| Hash | 1 | SHA-256 file hasher |
| Symmetric encryption | 2 | ChaCha20-Poly1305 |
| Signatures | 3 | Ed25519 sign/verify |
| Key exchange | 4 | X25519 Diffie-Hellman |
| Key derivation | 5 | HKDF from shared secret |
| Password KDFs | 6 | Argon2 / PBKDF2 |
| Certificates | 7 | X.509 parsing |
| Cert generation | 8 | Build a CA with rcgen |
| Mini-TLS | 9-12 | Encrypted echo server |
| TLS handshake | 13 | Protocol deep dive |
| Real TLS | 14-15 | `tokio-rustls` + HTTPS |

Every concept in this lesson maps to a specific part of TLS. The following lessons implement each piece in Rust.

## Check your understanding

Use these as self-check questions. Try answering before opening the answer.

<details>
<summary>Why is encryption alone not enough for secure communication?</summary>

Encryption provides confidentiality: it hides the message content. It does not automatically prove who sent the message, whether the message was modified, or whether an old valid message was replayed. TLS needs authentication, integrity, and replay protection in addition to encryption.

</details>

<details>
<summary>What is the difference between confidentiality and integrity?</summary>

Confidentiality means an attacker cannot read the data. Integrity means an attacker cannot change the data without detection. Encrypted data can still be tampered with unless the scheme also authenticates it, which is why modern TLS uses AEAD ciphers.

</details>

<details>
<summary>What does authentication prove in TLS?</summary>

Authentication proves that the peer owns the expected private key. For a normal HTTPS connection, the server presents a certificate and proves possession of the matching private key. The certificate chain links that key to a domain name through a trusted CA.

</details>

<details>
<summary>Why does TLS use a key exchange instead of just sending an encryption key?</summary>

If Alice sends the encryption key directly, anyone who can read that message can decrypt the session. A Diffie-Hellman style key exchange lets both sides derive the same shared secret without sending the secret itself over the network.

</details>

<details>
<summary>What is forward secrecy?</summary>

Forward secrecy means that stealing the server's long-term private key later does not decrypt old captured sessions. TLS gets this by using fresh ephemeral key exchange values for each session, then discarding the session secrets.

</details>

<details>
<summary>Why does TLS derive multiple keys from one shared secret? (covered in Lesson 5)</summary>

Different protocol directions and purposes need separate keys. For example, client-to-server traffic and server-to-client traffic should not reuse the same AEAD key. HKDF turns one shared secret into multiple independent keys with clear labels. You will implement this in Lesson 5.

</details>

<details>
<summary>What is a MAC, and how is it different from a plain hash or a digital signature?</summary>

These three constructions are easy to confuse because they all produce a short tag from a message:

|                  | Hash            | MAC                    | Digital signature        |
|------------------|-----------------|------------------------|--------------------------|
| **Key**          | none            | shared symmetric key   | asymmetric key pair      |
| **Who can produce** | anyone       | anyone with the key    | private key holder only  |
| **Who can verify**  | anyone       | anyone with the key    | anyone with the public key |
| **Proves**       | integrity (if hash is trusted) | integrity + "a key holder made it" | integrity + "this specific party signed" |
| **Non-repudiation** | no          | no (both sides have the key) | yes                   |
| **Speed**        | fast            | fast                   | slower                   |
| **Used in TLS**  | transcript, HKDF, HMAC | record integrity (AEAD tag) | handshake authentication |

TLS uses all three: hashes inside HMAC and HKDF, MACs inside AEAD for record integrity, and signatures during the handshake to authenticate the server.

</details>

<details>
<summary>What does AEAD give you?</summary>

AEAD means authenticated encryption with associated data. It encrypts the plaintext and also produces an authentication tag. During decryption, tampering is detected before the plaintext is accepted.

</details>

<details>
<summary>Why can a replay attack work even if the message is encrypted?</summary>

The attacker does not need to understand the message. If the same encrypted bytes are still valid later, the attacker can resend them. Protocols prevent this with sequence numbers, nonces, timestamps, or state that rejects duplicates.

</details>

<details>
<summary>What is the basic idea of a man-in-the-middle attack?</summary>

Mallory places herself between Alice and Bob, pretending to be Bob to Alice and Alice to Bob. Encryption alone does not stop this if Alice has no way to authenticate Bob's key. Certificates and handshake signatures are what bind the key to the real server identity.

</details>

<details>
<summary>What does the TLS Finished message protect? (covered in Lesson 13)</summary>

The Finished message authenticates the handshake transcript. It proves both sides saw the same negotiation, certificates, key exchange values, and parameters. If an attacker changed anything during the handshake, the Finished verification fails. You will implement this in Lesson 13 — try reasoning from the concepts in this lesson first.

</details>

<details>
<summary>What happens if you encrypt two messages with the same key and nonce?</summary>

With a stream cipher or GCM mode, the keystream is derived from the key and nonce together. If the nonce repeats, both messages are XORed against the identical keystream. An attacker who sees both ciphertexts can XOR them together, cancelling the keystream and recovering information about both plaintexts. For GCM specifically, nonce reuse also destroys the authentication tag, enabling forgery. This is why TLS uses a per-record sequence number as the nonce — it is strictly monotonic and never repeats.

</details>

<details>
<summary>Which TLS properties protect against passive and active attackers?</summary>

Encryption protects against passive eavesdropping. Authentication, integrity checks, transcript binding, and replay defenses protect against active attackers who modify, impersonate, downgrade, or resend messages.

</details>
