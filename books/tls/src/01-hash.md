# Lesson 1: Hashing (SHA-256)

## What is a hash function?

A hash function takes any input — a single byte, a password, an entire movie file — and produces a fixed-size output called a **digest**. SHA-256 always outputs 256 bits (32 bytes), no matter what you feed it.

```
"hello"           → 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
"hello "          → 98ea6e4f216f2fb4b69fff9b3a44842c38686ca685f3f55dc48c5d3fb1107be4
(entire Linux kernel) → some 32-byte value
```

## Properties

1. **Deterministic**: Same input always gives the same hash.
2. **One-way**: Given a hash, you can't recover the original input. You'd have to brute-force every possible input.
3. **Avalanche effect**: Change one bit → completely different hash. "hello" vs "hello " (one extra space) produce unrelated outputs.
4. **Collision resistant**: Practically impossible to find two different inputs with the same hash. 2^256 possible outputs — more than atoms in the observable universe.

## Real-world scenarios

### Alice verifies a downloaded file

Alice downloads a Linux ISO from a mirror. How does she know the file wasn't tampered with during transit?

1. The official website publishes: `ubuntu.iso SHA-256: a1b2c3d4...`
2. Alice downloads the ISO from a mirror
3. Alice runs `sha256sum ubuntu.iso` and gets `a1b2c3d4...`
4. The hashes match → the file is intact

If an attacker modified even one byte of the ISO, the hash would be completely different.

### Bob stores passwords safely

Bob runs a website. He never stores passwords in plaintext — if his database leaks, all passwords would be exposed.

1. User signs up with password "hunter2"
2. Bob stores `SHA-256("hunter2" + salt)` = `9f4e...` in the database
3. At login, Bob hashes the submitted password and compares with the stored hash
4. Bob never sees or stores the actual password

(In practice, you'd use bcrypt/argon2 instead of raw SHA-256 for passwords, because they're intentionally slow to prevent brute-force attacks.)

### How TLS uses hashing

TLS uses hashes everywhere:
- **Handshake transcript**: Both sides hash all handshake messages. If an attacker modified anything mid-handshake, the hashes won't match and the connection fails.
- **HMAC**: Hash + secret key = message authentication code (Lesson 5).
- **HKDF**: Derive encryption keys from a shared secret using hashing (Lesson 5).
- **Certificate fingerprints**: Identify certificates by their hash for pinning.

## Exercises

### Exercise 1: File hasher (implemented in 1-hash.rs)
Build a CLI tool that takes a file path and prints its SHA-256 hash. Verify against `shasum -a 256`.

### Exercise 2: Avalanche effect (try it yourself)
Hash these two strings and compare the outputs:
```
"The quick brown fox jumps over the lazy dog"
"The quick brown fox jumps over the lazy dog."
```
Only one character difference (added period), but the hashes are completely unrelated.

### Exercise 3: Hash chain
Compute `SHA-256(SHA-256(SHA-256("hello")))` — hash the hash of the hash. This is how Bitcoin's proof-of-work operates (double SHA-256). Each step is deterministic, so anyone can verify the chain.

### Exercise 4: Commitment scheme
Imagine Alice wants to prove she predicted the outcome of an event without revealing her prediction in advance:
1. Alice hashes her prediction: `h = SHA-256("team A wins")`
2. Alice publishes `h` before the event
3. After the event, Alice reveals "team A wins"
4. Everyone computes `SHA-256("team A wins")` and verifies it matches `h`

Alice can't change her prediction after publishing the hash (collision resistance), and nobody can figure out her prediction from the hash (one-way).
