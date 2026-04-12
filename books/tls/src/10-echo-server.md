# Lesson 10: Authenticated Echo Server

> **Alice's Bookstore — Chapter 10**
>
> Alice's encrypted channel from Lesson 9 is working. Customers can communicate securely. Then Mallory strikes:
>
> Mallory sets up a fake server at `alices-b00kstore.com` (with zeros instead of o's). When a customer connects, Mallory does her own DH key exchange with them. The customer thinks they're talking to Alice — the connection is encrypted, everything looks fine — but Mallory is reading every message.
>
> *"I thought encryption solved this!"*
>
> Bob: *"Encryption protects the pipe, but doesn't prove who's on the other end. You need to SIGN your DH public key so the customer can verify it's really you. Mallory can't forge your signature."*
>
> *"So the server proves its identity during the handshake?"*
>
> *"Exactly. That's authentication."*

## Real-life analogy: the phone call with caller ID

```
Without authentication (Lesson 9):
  Phone rings: "Hi, this is your bank. What's your account number?"
  You: "Sure, it's 12345"         ← could be a scammer!

With authentication (this lesson):
  Phone rings: "Hi, this is your bank"
  You: "Prove it. What's my security question?"
  Caller: "Your mother's maiden name is Smith"  ← only the real bank knows
  You: "OK, I trust you now"

In crypto terms:
  "Prove it"     = "sign your DH public key"
  "maiden name"  = the server's Ed25519 private key
  Verification   = checking the signature against a known public key
```

## The problem with Lesson 9

In Lesson 9, we built an encrypted channel. But the client has no way to verify who it's talking to. An attacker (Mallory) can sit between client and server, do separate DH key exchanges with each side, and read all traffic — a **man-in-the-middle attack**.

```
Client ←──DH──→ Mallory ←──DH──→ Server
        key_1             key_2

Mallory decrypts with key_1, reads, re-encrypts with key_2, forwards.
Neither side detects anything.
```

## The solution: sign the handshake

The server has a **long-term Ed25519 identity key pair** (generated once, stored on disk). The client knows the server's public key in advance. During the handshake, the server signs its ephemeral DH public key with its identity key. The client verifies the signature.

### The protocol (changes from Lesson 9 in bold)

```
Client                                     Server
  │                                          │
  │── client_dh_public (32 bytes) ─────────►│
  │                                          │
  │◄── server_dh_public (32 bytes) ─────────│
  │◄── signature (64 bytes) ────────────────│  ** sign(identity_key, server_dh_public) **
  │                                          │
  │  ** verify(known_pubkey,                 │
  │     server_dh_public, signature) **      │
  │  ** → if fails, ABORT **                 │
  │                                          │
  │  shared = DH(my_secret, their_public)    │  (same as Lesson 9)
  │  derive keys, encrypt/decrypt            │
```

Only 64 bytes more on the wire. But now Mallory can't impersonate the server.

## Why Mallory can't attack this

1. Mallory intercepts Alice's DH public key
2. Mallory generates her own DH key pair, sends `mallory_dh_pub` to Alice
3. Mallory needs to send a valid signature: `sign(server_identity_private, mallory_dh_pub)`
4. **Mallory doesn't have `server_identity_private`** — she can't forge the signature
5. Alice verifies the signature → **FAILS** → Alice disconnects

Mallory could sign with her own identity key, but Alice would reject it because Alice only trusts the server's known public key.

## Real-world scenarios

### SSH host verification

The first time you SSH to a server, you see:
```
The authenticity of host 'server.com' can't be established.
ED25519 key fingerprint is SHA256:abc123...
Are you sure you want to continue connecting (yes/no)?
```

You're manually deciding to trust this public key. Once you say "yes", it's saved in `~/.ssh/known_hosts`. On subsequent connections, SSH verifies the server's signature against the stored key. If it doesn't match:
```
WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!
```

This is exactly what Lesson 10 does — the client has a known server public key and verifies the handshake signature.

### TLS certificate verification

In real TLS, instead of a hardcoded public key, the server sends a **certificate** (Lesson 7) containing its public key, signed by a CA. The client verifies the certificate chain, extracts the public key, then verifies the handshake signature. Same principle, but with a trust hierarchy instead of a pinned key.

### WireGuard peer authentication

WireGuard uses the same pattern: each peer has a long-term X25519 key pair (Curve25519, not Ed25519, but same idea). You configure each peer with the other's public key. During the handshake, the Noise protocol uses static-ephemeral DH to authenticate — if you don't have the right private key, the handshake fails.

### Signal's safety numbers

When you verify "safety numbers" with a Signal contact, you're comparing long-term identity public keys. If the keys match, you know your messages are authenticated and not being intercepted. If Signal shows "safety number changed", the contact's identity key changed — could be a new phone, or could be a MITM.

## Try it yourself

```sh
# Step 1: generate the server's identity key
cargo run -p tls --bin 10-genkey
# Private key saved to server_identity.key
# Public key: dd8c3c76bf81163f...

# Step 2: start the authenticated server
cargo run -p tls --bin 10-echo-server

# Step 3: connect with the client (hardcode the public key from step 1)
cargo run -p tls --bin 10-echo-client
# "server authenticated" → type messages, see them echoed

# Step 4: test with wrong key — change one hex digit in the client
# Run again → "server authentication failed!" → connection refused
```

```sh
# See SSH doing the same thing:
# First connection to a new server:
ssh -v new-server.com 2>&1 | grep -i "host key"
# "Server host key: ssh-ed25519 SHA256:..."
# "Are you sure you want to continue connecting?"

# After accepting, it's in known_hosts:
grep "new-server" ~/.ssh/known_hosts

# If the server's key changes (or MITM):
# "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!"
# Same concept as our authenticated echo server.
```

## What this does NOT protect against

- **Compromised server**: If the attacker steals the server's private identity key, they can impersonate the server. This is why key storage matters (file permissions, HSMs in production).
- **Compromised client**: If the attacker modifies the client's known public key, they can substitute their own. This is why the trust anchor must be distributed securely.
- **Traffic analysis**: Authentication doesn't hide metadata (timing, message sizes, IP addresses).

## The three binaries

### 10-genkey.rs (run once)
Generates an Ed25519 identity key pair. Saves the private key to `server_identity.key`. Prints the public key as hex for the client to use.

### 10-echo-server.rs
Same as Lesson 9, plus:
- Loads the identity private key from `server_identity.key`
- After sending its DH public key, signs it and sends the 64-byte signature

### 10-echo-client.rs
Same as Lesson 9, plus:
- Has the server's public key hardcoded (or as a CLI argument)
- After receiving the DH public key and signature, verifies the signature
- If verification fails, disconnects immediately

## Comparison with real TLS

| Feature | Lesson 9 | Lesson 10 | TLS 1.3 |
|---------|----------|----------|---------|
| Key exchange | X25519 | X25519 | X25519 or P-256 |
| Server auth | None | Ed25519 signature | RSA/ECDSA/Ed25519 via certificate |
| Trust model | None | Pinned public key | CA hierarchy |
| Client auth | None | None | Optional (mutual TLS) |
| What's signed | Nothing | DH public key | Full handshake transcript |

In real TLS, the server signs the entire **handshake transcript** (all messages exchanged so far), not just the DH public key. This binds the signature to the entire handshake context, preventing more subtle attacks like transcript manipulation.

## Exercises

### Exercise 1: Authenticated echo (implemented in 10-echo-server.rs and 10-echo-client.rs)
Extend Lesson 9 with server authentication. Generate identity keys, sign the DH public key, verify on the client.

### Exercise 2: Test with wrong key
Change one byte of the hardcoded public key in the client. Run it — it should fail with a verification error, proving that authentication works.

### Exercise 3: Mutual authentication
Add client authentication too: the client also has a long-term Ed25519 key pair, and signs its DH public key. The server verifies. Now both sides know who they're talking to. This is how mutual TLS (mTLS) works — common in service-to-service communication.

### Exercise 4: Sign the transcript, not just the DH key
Instead of signing only the server's DH public key, sign the concatenation of both DH public keys: `sign(identity_key, client_dh_pub || server_dh_pub)`. This binds the signature to the entire key exchange, preventing an attacker from reusing a captured signature with a different client. This is closer to what real TLS does.
