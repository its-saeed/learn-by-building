# Lesson 5: HMAC and Key Derivation (HKDF)

## The problem

In Lesson 4, you got a 32-byte shared secret via DH. But you need **multiple** independent keys:
- One key for client → server encryption
- One key for server → client encryption
- (In real TLS: also keys for IVs, handshake encryption, resumption, etc.)

You can't reuse the same key for both directions. If you do, an attacker can reflect your own encrypted messages back to you and you'd accept them as valid.

Also, the raw DH shared secret has mathematical structure — it's a point on an elliptic curve, not uniformly random bytes. You want to "clean it up" into proper key material.

## HMAC: Hash-based Message Authentication Code

Before HKDF, you need to understand HMAC. It's simple:

```
HMAC(key, message) = Hash((key ⊕ opad) || Hash((key ⊕ ipad) || message))
```

In plain English: hash the message with the key mixed in, twice. The result is a fixed-size tag that proves:
1. **Integrity**: the message wasn't modified
2. **Authenticity**: only someone with the key could produce this tag

Unlike a plain hash, HMAC requires a secret. An attacker can compute `SHA-256("transfer $1000")` but can't compute `HMAC(secret, "transfer $1000")` without the key.

## HKDF: Extract and Expand

HKDF uses HMAC to derive keys in two steps:

### Step 1: Extract
```
PRK = HKDF-Extract(salt, input_key_material)
    = HMAC(salt, shared_secret)
```

Takes the raw DH output (which may have non-uniform randomness) and concentrates the entropy into a pseudorandom key (PRK). The salt is optional — even an empty salt works.

### Step 2: Expand
```
key_1 = HKDF-Expand(PRK, info="client-to-server", length=32)
key_2 = HKDF-Expand(PRK, info="server-to-client", length=32)
```

Takes the PRK and stretches it into multiple independent keys. The `info` parameter is a label — same PRK with different labels produces completely unrelated keys. You can generate as many keys as you need.

## Real-world scenarios

### Alice and Bob derive session keys

Continuing from Lesson 4: Alice and Bob have a shared DH secret.

1. Both compute: `PRK = HKDF-Extract(salt="tls13", shared_secret)`
2. Alice derives: `c2s_key = HKDF-Expand(PRK, "client-to-server", 32)`
3. Alice derives: `s2c_key = HKDF-Expand(PRK, "server-to-client", 32)`
4. Bob derives the exact same two keys (same PRK, same labels)
5. Alice encrypts messages TO Bob with `c2s_key`
6. Alice decrypts messages FROM Bob with `s2c_key`
7. Bob does the reverse

Even though both keys came from one shared secret, they're cryptographically independent. Compromising `c2s_key` doesn't reveal `s2c_key`.

### TLS 1.3 key schedule

TLS 1.3 uses HKDF extensively. The key schedule derives dozens of keys from the DH shared secret:

```
DH shared secret
  │
  ├─ HKDF → handshake_secret
  │           ├─ HKDF → client_handshake_key (encrypts ClientFinished)
  │           └─ HKDF → server_handshake_key (encrypts ServerFinished)
  │
  └─ HKDF → master_secret
              ├─ HKDF → client_application_key (encrypts app data c→s)
              ├─ HKDF → server_application_key (encrypts app data s→c)
              └─ HKDF → resumption_secret (for session resumption)
```

Each key has a unique label, so they're all independent. If one key leaks, the others remain secure.

### API token derivation

A web service needs to generate unique API tokens for each user from a master secret:

```
master = random 32 bytes (stored securely on server)
token_alice = HKDF-Expand(master, "user:alice", 32)
token_bob   = HKDF-Expand(master, "user:bob", 32)
```

Each token is unique and unpredictable, but the server only stores one master secret. If Alice's token is compromised, Bob's is safe — they're independent.

## HMAC vs plain hash: why it matters

Imagine Alice sends Bob a message with a checksum: `("transfer $100", SHA-256("transfer $100"))`. Eve intercepts it, computes `SHA-256("transfer $999")`, and replaces the checksum. Bob sees a valid checksum and processes the transfer.

With HMAC: Alice sends `("transfer $100", HMAC(shared_key, "transfer $100"))`. Eve can't forge the HMAC without the shared key. She can't even verify her forgery. Bob checks the HMAC → forgery is detected.

## Exercises

### Exercise 1: Derive two keys (implemented in 5-kdf.rs)
Take a shared secret, use HKDF to derive two 32-byte keys with different info strings. Print both — they must be different.

### Exercise 2: Deterministic derivation
Run the program twice with the same hardcoded shared secret and salt. Verify you get the exact same derived keys both times. This is critical — both sides of a TLS connection must derive identical keys independently.

### Exercise 3: HMAC verification
Use the `hmac` crate to compute `HMAC-SHA256(key, message)`. Then verify it:
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
type HmacSha256 = Hmac<Sha256>;

let mut mac = HmacSha256::new_from_slice(key)?;
mac.update(message);
let tag = mac.finalize().into_bytes();

// Verify
let mut mac = HmacSha256::new_from_slice(key)?;
mac.update(message);
mac.verify_slice(&tag)?; // Constant-time comparison!
```

Note: `verify_slice` uses constant-time comparison to prevent timing attacks. Never use `==` to compare MACs.

### Exercise 4: Full pipeline
Combine Lessons 4 + 5: do a DH key exchange, then derive two keys via HKDF, then encrypt a message with `c2s_key` (Lesson 2) and decrypt with the same key on the "other side". This is the core of what Lesson 7 will build over TCP.
