# Project: HTTPS Server

> **Prerequisites**: Lesson 8 (Certificate Generation), Lesson 14 (tokio-rustls). Serve a web page over TLS.

## What is this?

Every time you visit a website with the padlock icon, you're using HTTPS — HTTP over TLS. You're building the server side: accept browser connections, do the TLS handshake, serve HTML.

```
┌──────────────────────────────────────────────────────────┐
│  What happens when you type https://localhost:8443       │
│                                                          │
│  1. Browser connects via TCP to port 8443                │
│  2. TLS handshake (your cert, key exchange, encryption)  │
│  3. Browser sends: GET / HTTP/1.1\r\n                    │
│  4. Your server responds: 200 OK + HTML                  │
│  5. Browser renders the page + shows padlock 🔒          │
│                                                          │
│  Without TLS (plain HTTP):                               │
│    Same thing, but no encryption.                        │
│    Anyone on the network sees the HTML and all data.     │
│    Browser shows "Not Secure" ⚠️                         │
└──────────────────────────────────────────────────────────┘
```

## What you're building

```sh
cargo run -p tls --bin p6-https-server
# Listening on https://127.0.0.1:8443

# Open in browser: https://127.0.0.1:8443
# → TLS handshake → padlock icon → your HTML page
```

## Try it with existing tools first

```sh
# === Python: HTTPS server in 3 lines ===
# First, generate a cert:
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout server.key -out server.crt -days 365 -subj "/CN=localhost"

# Start an HTTPS server:
python3 -c "
import http.server, ssl
server = http.server.HTTPServer(('127.0.0.1', 8443), http.server.SimpleHTTPRequestHandler)
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.load_cert_chain('server.crt', 'server.key')
server.socket = ctx.wrap_socket(server.socket, server_side=True)
print('Listening on https://127.0.0.1:8443')
server.serve_forever()
"

# Test with curl (skip cert verification for self-signed):
curl -k https://127.0.0.1:8443/
# Shows directory listing

# Test with openssl:
echo "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n" | \
  openssl s_client -connect 127.0.0.1:8443 -quiet 2>/dev/null
```

## Architecture

```
Browser                          Your HTTPS Server
  │                                    │
  ├── TCP connect :8443 ─────────────►│
  │                                    │
  │◄── TLS handshake ────────────────►│  rustls handles:
  │    ClientHello / ServerHello       │  - certificate
  │    Certificate / Finished          │  - key exchange
  │                                    │  - encryption setup
  │                                    │
  │── GET / HTTP/1.1\r\n ───────────►│  encrypted inside TLS
  │   Host: localhost\r\n              │
  │   \r\n                             │
  │                                    │
  │◄── HTTP/1.1 200 OK\r\n ─────────│  your response
  │    Content-Type: text/html\r\n     │  (also encrypted)
  │    Content-Length: 45\r\n           │
  │    \r\n                            │
  │    <h1>Hello from Rust!</h1>       │
```

## Implementation guide

### Step 0: Project setup

```sh
touch tls/src/bin/p6-https-server.rs
```

Add to `tls/Cargo.toml`:

```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "io-util"] }
tokio-rustls = "0.26"
rustls = "0.23"
rcgen = "0.13"
```

### Step 1: Generate a self-signed certificate (in code)

No openssl CLI needed — use rcgen from Lesson 8:

```rust
use rcgen::generate_simple_self_signed;

fn generate_cert() -> (Vec<u8>, Vec<u8>) {
    let cert = generate_simple_self_signed(vec![
        "localhost".into(),
        "127.0.0.1".into(),
    ]).unwrap();

    let cert_der = cert.cert.der().to_vec();
    let key_der = cert.key_pair.serialize_der();

    println!("Generated self-signed cert for localhost");
    (cert_der, key_der)
}
```

Test: print the cert PEM and inspect with openssl:

```rust
println!("{}", cert.cert.pem());
// Save to cert.pem, then:
// openssl x509 -in cert.pem -text -noout
```

### Step 2: Configure rustls server

```rust
use std::sync::Arc;
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

fn make_tls_config(cert_der: Vec<u8>, key_der: Vec<u8>) -> Arc<ServerConfig> {
    let certs = vec![CertificateDer::from(cert_der)];
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("bad cert/key");

    Arc::new(config)
}
```

### Step 3: Accept TLS connections

```rust
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

#[tokio::main]
async fn main() {
    let (cert_der, key_der) = generate_cert();
    let tls_config = make_tls_config(cert_der, key_der);
    let acceptor = TlsAcceptor::from(tls_config);

    let listener = TcpListener::bind("127.0.0.1:8443").await.unwrap();
    println!("Listening on https://127.0.0.1:8443");

    loop {
        let (tcp_stream, addr) = listener.accept().await.unwrap();
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            match acceptor.accept(tcp_stream).await {
                Ok(mut tls_stream) => {
                    println!("[{addr}] TLS handshake complete");
                    handle_request(&mut tls_stream).await;
                }
                Err(e) => eprintln!("[{addr}] TLS error: {e}"),
            }
        });
    }
}
```

Test: the server starts. Connect with curl:

```sh
curl -k https://127.0.0.1:8443/
# Hangs — because handle_request is not implemented yet
```

### Step 4: Parse HTTP request and respond

HTTP/1.1 is just text over TCP (which is now TLS):

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::server::TlsStream;
use tokio::net::TcpStream;

async fn handle_request(stream: &mut TlsStream<TcpStream>) {
    // Read the HTTP request
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).await.unwrap_or(0);
    if n == 0 { return; }

    let request = String::from_utf8_lossy(&buf[..n]);
    let first_line = request.lines().next().unwrap_or("");
    println!("  Request: {first_line}");

    // Route
    let (status, body) = if first_line.starts_with("GET / ") {
        ("200 OK", "<html><body><h1>Hello from Rust HTTPS!</h1><p>Your connection is encrypted.</p></body></html>")
    } else if first_line.starts_with("GET /about") {
        ("200 OK", "<html><body><h1>About</h1><p>Built with tokio-rustls.</p></body></html>")
    } else {
        ("404 Not Found", "<html><body><h1>404</h1><p>Page not found.</p></body></html>")
    };

    // Send HTTP response
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await.ok();
}
```

### Step 5: Test it

```sh
# Start the server:
cargo run -p tls --bin p6-https-server

# Test with curl:
curl -k https://127.0.0.1:8443/
# <html><body><h1>Hello from Rust HTTPS!</h1>...

curl -k https://127.0.0.1:8443/about
# <html><body><h1>About</h1>...

curl -k https://127.0.0.1:8443/nonexistent
# <html><body><h1>404</h1>...

# Test with openssl (see the TLS details):
echo -e "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n" | \
  openssl s_client -connect 127.0.0.1:8443 -quiet 2>/dev/null

# Open in browser:
# https://127.0.0.1:8443
# You'll see a security warning (self-signed cert) — click through it.
# The page appears. The padlock shows it's encrypted.
```

```sh
# See the TLS details from the server's perspective:
# Add to your handle_request:
let (_, conn) = stream.get_ref();
println!("  Protocol: {:?}", conn.protocol_version());
println!("  Cipher:   {:?}", conn.negotiated_cipher_suite());
```

## What curl's `-k` flag does

```
Without -k:
  curl https://127.0.0.1:8443/
  → ERROR: self-signed certificate
  curl checks the cert against trusted CAs — yours isn't trusted.

With -k (--insecure):
  curl -k https://127.0.0.1:8443/
  → Works! curl skips certificate verification.
  The connection is still encrypted — just not authenticated.

With your CA cert:
  curl --cacert ca.crt https://127.0.0.1:8443/
  → Works AND verified! No warning.
  (Requires generating a CA cert and signing your server cert with it — Lesson 8)
```

## Exercises

### Exercise 1: Basic HTTPS server

Implement steps 1-5. Serve a static HTML page. Verify with `curl -k`.

### Exercise 2: Serve static files

Serve files from a `./public/` directory:
- `GET /index.html` → read `./public/index.html`
- `GET /style.css` → read `./public/style.css`
- Set `Content-Type` based on file extension (html, css, js, png, etc.)

### Exercise 3: CA-signed certificate

Instead of self-signed, generate a CA cert + server cert (Lesson 8). Install the CA cert on your system:

```sh
# macOS:
sudo security add-trusted-cert -d -r trustRoot \
  -k /Library/Keychains/System.keychain ca.crt

# Now curl works WITHOUT -k:
curl https://127.0.0.1:8443/
# No warning! Proper padlock in browser too.

# Clean up:
sudo security remove-trusted-cert -d ca.crt
```

### Exercise 4: Request logging

Log each request with: timestamp, client IP, method, path, response status, TLS version, cipher suite.

```
[2026-04-13 10:30:00] 127.0.0.1 GET / → 200 (TLS 1.3, AES-256-GCM)
[2026-04-13 10:30:01] 127.0.0.1 GET /about → 200 (TLS 1.3, AES-256-GCM)
[2026-04-13 10:30:02] 127.0.0.1 GET /missing → 404 (TLS 1.3, AES-256-GCM)
```
