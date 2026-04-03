// Lesson 4: Diffie-Hellman Key Exchange (X25519)
//
// Problem: two parties need a shared secret, but can't send it in plaintext.
//
// Solution (Diffie-Hellman):
//   1. Alice generates ephemeral secret + public key
//   2. Bob generates ephemeral secret + public key
//   3. They exchange public keys (safe to send in the open)
//   4. Alice: shared = alice_secret * bob_public
//      Bob:   shared = bob_secret * alice_public
//   5. Both arrive at the SAME shared secret
//
// An eavesdropper sees both public keys but can't compute the shared secret
// without one of the private keys (elliptic curve discrete log problem).
//
// X25519: elliptic curve DH on Curve25519. 32-byte keys, 32-byte shared secret.
//
// Forward secrecy: keys are ephemeral (new each connection). Even if a long-term
// key is later compromised, past sessions can't be decrypted because the
// ephemeral keys were destroyed. TLS 1.3 mandates this.

use x25519_dalek::{EphemeralSecret, PublicKey};

fn main() {
    let alice_secret = EphemeralSecret::random_from_rng(rand_core::OsRng);
    let bob_secret = EphemeralSecret::random_from_rng(rand_core::OsRng);
    let alice_pub = PublicKey::from(&alice_secret);
    let bob_pub = PublicKey::from(&bob_secret);
    let shared_key1 = alice_secret.diffie_hellman(&bob_pub);
    let shared_key2 = bob_secret.diffie_hellman(&alice_pub);
    println!("{}\n{}", hex::encode(shared_key1.as_bytes()), hex::encode(shared_key2.as_bytes()))
}
