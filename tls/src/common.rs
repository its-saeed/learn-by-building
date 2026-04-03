use std::{io::{self, Read, Write}, net::TcpStream};

use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce, aead::Aead};
use hkdf::Hkdf;
use sha2::Sha256;

pub fn derive_keys(shared_secret: &[u8]) -> (ChaCha20Poly1305, ChaCha20Poly1305) {
    let kdf = Hkdf::<Sha256>::new(Some(b"salam"), shared_secret);
    let mut c2s_key = [0u8; 32];
    let mut s2c_key = [0u8; 32];
    kdf.expand(b"c2s", &mut c2s_key).unwrap();
    kdf.expand(b"s2c", &mut s2c_key).unwrap();
    (ChaCha20Poly1305::new(Key::from_slice(&c2s_key)),
        ChaCha20Poly1305::new(Key::from_slice(&s2c_key)))
}

pub fn send_encrypted(stream: &mut TcpStream, cipher: &ChaCha20Poly1305, msg: &[u8]) {
    let nonce: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce);
    let ciphertext = cipher.encrypt(&nonce, msg).unwrap();
    let total_length = 12 + ciphertext.len();
    stream.write_all(&(total_length as u16).to_be_bytes()).unwrap();
    stream.write_all(nonce.as_ref()).unwrap();
    stream.write_all(&ciphertext).unwrap();
}

pub fn recv_encrypted(stream: &mut TcpStream, cipher: &ChaCha20Poly1305) -> Result<Vec<u8>, io::Error> {
    let mut len = [0u8; 2];
    stream.read_exact(&mut len)?;
    let len = u16::from_be_bytes(len) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    let (nonce, ciphertext) = buf.split_at(12);
    let nonce = Nonce::from_slice(&nonce);
    Ok(cipher.decrypt(nonce, ciphertext).unwrap())
}
