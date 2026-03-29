// Lesson 3: Asymmetric Crypto & Signatures (Ed25519)
//
// Key pairs: private key (secret) + public key (shared with everyone).
//
// Digital signatures:
//   - Sign with private key → anyone with public key can verify
//   - Proves: "this message was created by the private key holder"
//   - Proves: "this message hasn't been modified since signing"
//
// Ed25519: modern elliptic curve signature scheme.
//   - 32-byte keys, 64-byte signatures, very fast
//   - Used by SSH, WireGuard, and TLS implementations
//
// In TLS: the server signs the handshake data with its long-term private key.
// The client verifies with the server's public key (from the certificate).
// This is how the client knows it's talking to the real server, not an impersonator.

use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};

fn main() {
    let mut signing_key: SigningKey = SigningKey::generate(&mut rand_core::OsRng);
    let signature = signing_key.sign(b"hello");
    println!("{}", hex::encode(signature.to_bytes()));
    let public_key = signing_key.verifying_key();
    match public_key.verify_strict(b"hello", &signature) {
        Ok(_) => println!("verified"),
        Err(_) => println!("not verfified"),
    }
    match public_key.verify_strict(b"helloo", &signature) {
        Ok(_) => println!("verified"),
        Err(_) => println!("not verfified"),
    }
}
