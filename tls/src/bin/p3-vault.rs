use std::collections::HashMap;

use argon2::Argon2;
use bytes::BytesMut;
use clap::{Parser, Subcommand};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce, aead::AeadMut};
use image::EncodableLayout;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "vault", about = "Encrypted password vault")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Path to the vault file
    #[arg(long, default_value = "vault.enc")]
    vault: String,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new empty vault
    Init,
    /// Add a new entry
    Add { name: String },
    /// Retrieve an entry
    Get { name: String },
    /// List all entry names
    List,
}

fn ask_password(prompt: &str) -> String {
    rpassword::prompt_password(prompt).unwrap()
}

fn ask_input(prompt: &str) -> String {
    eprint!("{prompt}");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn derive_key(password: &[u8], salt: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    Argon2::default().hash_password_into(password, salt, &mut key).unwrap();
    key
}

fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> ([u8; 12], Vec<u8>) {
    let mut cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce_bytes: [u8; 12] = rand::random();
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), plaintext).expect("encryption failed");
    (nonce_bytes, ciphertext)
}

fn decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    let mut cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| "Wrong password or corrupted vault".to_string())
}

fn save_vault(path: &str, salt: &[u8; 16], nonce: &[u8; 12], ciphertext: &[u8]) {
    let mut bytes = BytesMut::new();
    bytes.extend_from_slice(salt);
    bytes.extend_from_slice(nonce);
    bytes.extend_from_slice(ciphertext);
    std::fs::write(path, bytes.as_bytes()).unwrap();
}

fn load_vault(path: &str) -> ([u8; 16], [u8; 12], Vec<u8>) {
    let data = std::fs::read(path).expect("Can't read vault file");
    assert!(data.len() >= 28, "Vault file too small — corrupted?");

    let salt: [u8; 16] = data[..16].try_into().unwrap();
    let nonce: [u8; 12] = data[16..28].try_into().unwrap();
    let ciphertext = data[28..].to_vec();
    (salt, nonce, ciphertext)
}

#[derive(Serialize, Deserialize, Default)]
struct Vault {
    entries: HashMap<String, Entry>,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    username: String,
    password: String,
}

fn open_vault(path: &str) -> (Vault, [u8; 16]) {
    let (salt, nonce, ciphertext) = load_vault(path);
    let password = prompt_password("Enter master password: ").expect("master password not provided");
    let key = derive_key(password.as_bytes(), &salt);
    let plaintext = decrypt(&key, &nonce, &ciphertext.as_bytes()).expect("failed to decrypt");
    let vault: Vault = serde_json::from_slice(&plaintext.as_bytes()).expect("failed to parse json");
    (vault, salt)
}

fn save(path: &str, vault: &Vault, salt: &[u8; 16], password: &str) {
    let key = derive_key(password.as_bytes(), salt);
    let json = serde_json::to_vec(vault).expect("failed to serialize to json");
    let (nonce, ciphertext) = encrypt(&key, json.as_bytes());
    save_vault(path, salt, &nonce, &ciphertext);
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            let password = prompt_password("Master password: ").unwrap();
            let confirm = prompt_password("Confirm password: ").unwrap();

            if password != confirm {
                eprintln!("Passwords don't match!");
                return;
            }
            let salt: [u8; 16] = rand::random();
            let vault = Vault::default();
            save(&cli.vault, &vault, &salt, &password);
            println!("Created {}", cli.vault);
        },
        Command::Add { name } => {
            let password = ask_password("Master password: ");
            let (mut vault, salt) = open_vault(&cli.vault);
            let username = ask_input("Username: ");
            let entry_password = ask_input("Password: ");

            vault.entries.insert(name.clone(), Entry {
                username,
                password: entry_password,
            });
            save(&cli.vault, &vault, &salt, &password);
            println!("Saved entry: {name}");
        },
        Command::Get { name } => {
            let (vault, _) = open_vault(&cli.vault);
            match vault.entries.get(&name) {
                Some(entry) => {
                    println!("Username: {}", entry.username);
                    println!("Password: {}", entry.password);
                }
                None => eprintln!("Entry '{name}' not found"),
            }
        },
        Command::List => {
            let (vault, _) = open_vault(&cli.vault);
            for name in vault.entries.keys() {
                println!("{name}")
            }
        }
    }
}
