// Lesson 1: Hashing (SHA-256)
//
// A hash function takes any input and produces a fixed-size output (digest).
// SHA-256 always outputs 32 bytes, regardless of input size.
//
// Properties:
//   - Deterministic: same input → same hash
//   - One-way: can't recover input from hash
//   - Avalanche effect: one bit change → completely different hash
//   - Collision resistant: practically impossible to find two inputs with same hash
//
// TLS uses hashing for: integrity checks, HMAC, key derivation (HKDF),
// certificate fingerprints, and handshake transcript verification.

use sha2::Digest;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(long)]
    file_path: PathBuf
}

fn main() {
    let args = Args::parse();
    let file_content = std::fs::read(&args.file_path).unwrap();
    let mut hash = sha2::Sha256::new();
    hash.update(file_content);

    let hash_str = hex::encode(hash.finalize());
    println!("SHA-256 hash: {}", hash_str);
}
