// Lesson 2: Symmetric Encryption (ChaCha20-Poly1305)
//
// One shared key encrypts and decrypts. Fast — used for all bulk data in TLS.
//
// AEAD (Authenticated Encryption with Associated Data):
//   Encrypts AND authenticates in one step. Output = ciphertext + 16-byte tag.
//   If even one bit of ciphertext is modified, decryption fails (tamper detection).
//
// ChaCha20-Poly1305 (one of two ciphers in TLS 1.3):
//   - ChaCha20: stream cipher (encryption)
//   - Poly1305: MAC (authentication)
//
// Nonce (12 bytes): number used once. NEVER reuse with the same key.
//   Reusing a nonce lets an attacker XOR two ciphertexts to recover plaintext.
//   In TLS, the nonce is a simple counter (0, 1, 2, ...).

use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce, aead::Aead};


fn main() {
    let key: [u8; 32] = rand::random();
    let nonce: [u8; 12] = rand::random();
    let plaintext = b"Hello, world!";

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let nonce = Nonce::from_slice(&nonce);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).unwrap();
    println!("cipher: {}", hex::encode(&ciphertext));

    let plaintext_decrypted = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();
    println!("plaintext: {}", String::from_utf8_lossy(&plaintext_decrypted));

    let mut tampered = ciphertext.clone();
    tampered[0] ^= 0xFF; // flip one byte
    match cipher.decrypt(nonce, tampered.as_ref()) {
        Ok(_) => println!("decrypted (unexpected!)"),
        Err(e) => println!("tamper detected: {}", e),
    }
}
