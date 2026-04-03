# Lesson 4: Diffie-Hellman Key Exchange (X25519)

## The core problem

Alice and Bob want to encrypt their communication (Lesson 2), but they need a shared secret key. They can't send it in plaintext — Eve is watching the network. They can't encrypt it — that requires a key they don't have yet (chicken-and-egg).

## The magic trick (paint analogy)

1. Alice and Bob publicly agree on a base color: **yellow**
2. Alice picks a secret color: **red**. Mixes red + yellow → **orange**. Sends orange to Bob.
3. Bob picks a secret color: **blue**. Mixes blue + yellow → **green**. Sends green to Alice.
4. Alice mixes Bob's green + her secret red → **brown**
5. Bob mixes Alice's orange + his secret blue → **same brown**

Eve sees yellow, orange, and green — but can't unmix paint to get brown. That's Diffie-Hellman.

## The math (simplified)

With numbers and modular arithmetic:

```
Public parameters: p = 23 (prime), g = 5 (generator)

Alice                               Bob
picks secret a = 6                  picks secret b = 15

A = g^a mod p                      B = g^b mod p
A = 5^6 mod 23 = 8                 B = 5^15 mod 23 = 19

sends A = 8 ──────────────────►    receives A = 8
receives B = 19 ◄──────────────    sends B = 19

shared = B^a mod p                 shared = A^b mod p
       = 19^6 mod 23 = 2                 = 8^15 mod 23 = 2
```

Both get **2**. Why?
```
Alice: B^a = (g^b)^a = g^(b*a) mod p
Bob:   A^b = (g^a)^b = g^(a*b) mod p
a*b == b*a, so they're equal.
```

Eve sees g=5, p=23, A=8, B=19. To find the shared secret, she'd need to solve `5^a mod 23 = 8` for `a` — the **discrete logarithm problem**. With small numbers it's trivial, but with 256-bit numbers it's computationally infeasible.

## X25519: the modern version

Instead of `g^a mod p`, X25519 uses **elliptic curve point multiplication**:
- Secret key `a` = random 32 bytes
- Public key `A` = `a * G` (multiply base point G on Curve25519 by scalar a)
- Shared secret = `a * B` = `a * (b * G)` = `b * (a * G)` = `b * A`

Same principle, different math. Elliptic curves give equivalent security with much smaller keys (32 bytes vs 2048+ bytes for classic DH).

## Real-world scenarios

### Alice and Bob establish an encrypted chat session

Alice and Bob have never communicated before. They want to set up end-to-end encryption.

1. Alice generates an ephemeral X25519 key pair: `(alice_secret, alice_public)`
2. Bob generates an ephemeral X25519 key pair: `(bob_secret, bob_public)`
3. Alice sends `alice_public` (32 bytes) to Bob over the internet
4. Bob sends `bob_public` (32 bytes) to Alice over the internet
5. Alice computes: `shared = alice_secret.dh(bob_public)` → 32-byte secret
6. Bob computes: `shared = bob_secret.dh(alice_public)` → same 32-byte secret
7. Both use this shared secret as an encryption key (or derive keys via HKDF, Lesson 5)
8. Both destroy their ephemeral secrets

Eve recorded all traffic. She has `alice_public` and `bob_public`. She cannot compute the shared secret.

### Forward secrecy in TLS

Every TLS connection generates fresh ephemeral keys:

1. Monday: Client and server do DH → `shared_1`. Encrypt traffic. Destroy ephemeral keys.
2. Tuesday: Client and server do DH → `shared_2`. Encrypt traffic. Destroy ephemeral keys.
3. Wednesday: Attacker compromises the server's long-term private key.

The attacker recorded Monday's and Tuesday's encrypted traffic. Can they decrypt it? **No.** The ephemeral DH keys are gone. `shared_1` and `shared_2` can never be reconstructed. This is **forward secrecy**.

Without ephemeral DH (old RSA key exchange): the attacker uses the long-term key to decrypt ALL past traffic. This is why TLS 1.3 removed RSA key exchange entirely.

### WireGuard's Noise protocol

WireGuard uses X25519 for both:
- **Static keys**: long-term identity (like a certificate)
- **Ephemeral keys**: per-session (forward secrecy)

The handshake does multiple DH operations: static-static, static-ephemeral, ephemeral-ephemeral. This gives authentication AND forward secrecy in one round trip.

## The man-in-the-middle problem

DH alone does NOT authenticate. Mallory (attacker) can intercept:

```
Alice ←DH→ Mallory ←DH→ Bob
```

Mallory does two separate key exchanges. She decrypts Alice's messages, reads them, re-encrypts for Bob. Neither side knows. This is why Lessons 3 and 6 (signatures and certificates) are necessary — they authenticate the DH public keys.

## Exercises

### Exercise 1: Key exchange (implemented in 4-keyexchange.rs)
Simulate Alice and Bob. Generate ephemeral keys, exchange public keys, compute shared secrets. Print both — they must match.

### Exercise 2: Ephemeral means unique
Run the key exchange three times. Print the shared secret each time. All three should be different — demonstrating that each session gets a unique key.

### Exercise 3: Wrong public key
Alice does DH with Bob's public key. Charlie does DH with Bob's public key using a DIFFERENT secret. Show that Alice and Charlie get different shared secrets — only the matching pair produces the same result.

### Exercise 4: Simulate man-in-the-middle
Implement Mallory intercepting the exchange:
1. Alice generates keys, sends `alice_public` to Mallory (thinking it's Bob)
2. Mallory generates her own keys, sends `mallory_public` to Alice (pretending to be Bob)
3. Mallory sends `mallory_public2` to Bob (pretending to be Alice)
4. Bob sends `bob_public` to Mallory
5. Mallory now has two different shared secrets: one with Alice, one with Bob
6. Show that Alice's shared secret != Bob's shared secret (they're not talking to each other)
