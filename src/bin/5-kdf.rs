// Lesson 5: HMAC and Key Derivation (HKDF)
//
// Problem: the DH shared secret is one 32-byte value, but you need multiple
// independent keys (client→server, server→client, IVs, etc.). You also want
// to "clean up" the raw DH output into uniformly random key material.
//
// HKDF (HMAC-based Key Derivation Function) — two steps:
//
//   Extract: HKDF-Extract(salt, shared_secret) → PRK
//     Concentrates entropy into a pseudorandom key.
//
//   Expand: HKDF-Expand(PRK, info, length) → key material
//     Different "info" labels produce independent keys from the same PRK.
//     e.g. "client-to-server" → key_1, "server-to-client" → key_2
//
// Built on HMAC, which is built on hashing:
//   HMAC(key, msg) = Hash((key ^ opad) || Hash((key ^ ipad) || msg))
//   Unlike plain hash, requires a secret key → proves authenticity.
//
// TLS 1.3 uses HKDF extensively to derive all session keys from the
// handshake shared secret.

use hkdf::Hkdf;
use sha2::Sha256;

fn main() {
    let secret = hex::decode("bd60b1a90d98a3a49bd7abeab609f785f94ee890643004a3b686231d30835414").unwrap();
    let kdf = Hkdf::<Sha256>::new(Some(b"salam"), &secret);
    let mut c2s_key = [0u8; 32];
    let mut s2c_key = [0u8; 32];
    kdf.expand(b"client-to-server", &mut c2s_key).unwrap();
    kdf.expand(b"server-to-client", &mut s2c_key).unwrap();

    println!("c2s: {}", hex::encode(c2s_key));
    println!("s2c: {}", hex::encode(s2c_key));
}
