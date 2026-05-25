# Lesson 15: HTTPS Client

> **Alice's Bookstore — Chapter 15 (Finale)**
>
> Alice's bookstore is now running on real TLS with tokio-rustls. Bob suggests one final exercise:
>
> *"You've been building the server side. Now build the client side — connect to a real website over HTTPS, do the TLS handshake, send an HTTP request, get back HTML. When you see HTML from example.com appear in your terminal, you'll know that every concept from Lessons 1 through 14 just happened in about 50 milliseconds."*
>
> Alice builds it. She runs it. HTML appears. She stares at it for a moment.
>
> *"Hashing, encryption, signatures, key exchange, key derivation, certificates, authentication, replay defense... all of that just happened?"*
>
> *"All of it. In one handshake. And now you understand every layer."*
>
> Alice's bookstore is secure. Her customers' credit cards are safe. Eve can't read them. Mallory can't impersonate her. Replayed messages are rejected. The full circle is complete.

## Real-life analogy: making a phone call to a business

```
You call a company:
  1. You dial the number              → TCP connect to port 443
  2. Receptionist answers:            → TLS handshake begins
     "Company X, how can I help?"
  3. You verify it's really them      → certificate verification
     (caller ID, security question)
  4. You have a private conversation  → encrypted HTTP request/response
  5. You hang up                      → connection close
```

This lesson: you're building the phone. The network is the phone line. TLS is the encryption. HTTP is the conversation.

## The full circle

In Lesson 1, you hashed a file. Now you'll connect to a real website over TLS — using every concept you've learned. When you run this program and see HTML from `example.com`, know that under the hood:

1. Your client and the server exchanged X25519 keys (Lesson 4)
2. The server's certificate was verified against a CA (Lesson 7)
3. The server signed the handshake (Lessons 3 and 13)
4. Session keys were derived with HKDF (Lesson 5)
5. All data is encrypted with AES-GCM or ChaCha20-Poly1305 (Lesson 2)
6. Each record has a sequence number nonce (Lesson 12)
7. The hash of the handshake transcript ties it all together (Lesson 1)

## What HTTPS actually is

HTTPS = HTTP over TLS over TCP. That's it.

```
Application:  HTTP (GET /index.html, 200 OK, headers, body)
Security:     TLS  (handshake, encrypt, authenticate)
Transport:    TCP  (reliable byte stream)
Network:      IP   (routing)
```

The HTTP protocol doesn't change at all. You send the same `GET / HTTP/1.1\r\n` request. The only difference is that the TCP stream is wrapped in TLS, so everything is encrypted.

## Server Name Indication (SNI)

One IP address can host many HTTPS websites (virtual hosting). But TLS handshake happens before HTTP, so the server doesn't know which site you want yet. **SNI** solves this: the client sends the hostname in the ClientHello (plaintext).

```rust
let server_name = "example.com".try_into().unwrap();
connector.connect(server_name, tcp_stream).await?;
//                ^^^^^^^^^^^^ sent in ClientHello
```

The server uses SNI to pick the right certificate. This is why SNI is visible to network observers — it's the one piece of metadata that leaks in TLS (Encrypted Client Hello / ECH aims to fix this).

## Root certificate stores

Your browser and OS ship with ~150 trusted root CA certificates. These are the trust anchors for the entire web.

In Rust, you have two options:
- **`webpki-roots`**: Mozilla's root store compiled into your binary. No system dependency.
- **`rustls-native-certs`**: Loads from the OS certificate store. Respects system-level CA additions/removals.

For a simple client, `webpki-roots` is easiest.

## Real-world scenarios

### What your browser does thousands of times a day

Every time you visit a website:
1. DNS lookup → IP address
2. TCP connect to port 443
3. TLS handshake (what you built in Lessons 4-8, plus certificate chain validation from Lesson 7)
4. Send HTTP request through the encrypted tunnel
5. Receive HTTP response
6. Render HTML

Your browser does this in ~100ms. The TLS handshake is typically 1 RTT (TLS 1.3) or 2 RTT (TLS 1.2).

### curl under the hood

When you run `curl https://example.com`, it does exactly what this lesson implements:
1. Connects TCP to example.com:443
2. Does a TLS handshake (using OpenSSL or rustls depending on build)
3. Sends `GET / HTTP/1.1\r\nHost: example.com\r\n\r\n`
4. Prints the response body

You're building curl's core networking in ~30 lines of Rust.

### API clients

Every REST API call over HTTPS follows this pattern. When your code calls `reqwest::get("https://api.github.com/users/octocat")`, it's doing a TLS handshake, sending HTTP, and parsing the response — exactly what you're implementing here.

## What you'll see in the output

```
HTTP/1.1 200 OK
Content-Type: text/html; charset=UTF-8
Content-Length: 1256
...

<!doctype html>
<html>
<head>
    <title>Example Domain</title>
...
```

Real HTML, fetched over real TLS, verified against real CA certificates.

## TLS debugging cookbook

Most TLS bugs are not mysterious. They usually fall into one of a few buckets: name mismatch, expired certificate, untrusted CA, missing intermediate, wrong SNI, protocol mismatch, or a local clock problem.

### Start with curl

```sh
curl -v https://example.com/
```

Look for:

```
* Server certificate:
*  subject: CN=example.com
*  start date: ...
*  expire date: ...
*  subjectAltName: host "example.com" matched cert's "example.com"
*  issuer: ...
*  SSL certificate verify ok.
```

Common failures:

| Error | Meaning | Fix |
|---|---|---|
| `certificate has expired` | The certificate is past `notAfter` | Renew and reload the server |
| `no alternative certificate subject name matches` | The SAN list does not include the hostname | Issue a cert with the correct DNS/IP SAN |
| `self signed certificate` | The client does not trust the issuer | Use a public CA or pass/install the private CA |
| `unable to get local issuer certificate` | The server probably omitted an intermediate | Serve the full certificate chain |
| `wrong version number` | Client spoke TLS to a plaintext port, or proxy routing is wrong | Check port, scheme, and proxy config |

### Inspect the chain with OpenSSL

```sh
openssl s_client -connect example.com:443 -servername example.com -showcerts
```

Use `-servername` deliberately. Without SNI, many shared servers return a default certificate that does not match the hostname.

Check the leaf certificate:

```sh
echo | openssl s_client -connect example.com:443 -servername example.com 2>/dev/null | \
  openssl x509 -noout -subject -issuer -dates -ext subjectAltName
```

Check verification with an explicit CA file:

```sh
openssl verify -CAfile ca.crt server.crt
curl --cacert ca.crt https://localhost:8443/
```

If `curl -k` works but plain `curl` fails, encryption probably works but authentication does not. Fix trust, hostname, expiry, or chain configuration instead of leaving verification disabled.

### Debug rustls clients

When a `tokio-rustls` client fails, map the error back to the same checks:

| rustls/client symptom | Likely cause |
|---|---|
| `InvalidCertificate(Expired)` | Server certificate expired or local clock is wrong |
| `InvalidCertificate(NotValidForName)` | SNI/hostname does not match certificate SAN |
| `InvalidCertificate(UnknownIssuer)` | Root CA missing from the client's root store |
| Handshake fails only by IP address | Certificate has DNS SANs but no IP SAN |
| Works with `webpki-roots`, fails with native roots | OS trust store differs from Mozilla roots |
| Works locally, fails in container | Container lacks expected CA bundle or time sync |

For local development, prefer `curl --cacert ca.crt` and a client `RootCertStore` containing your test CA. Avoid disabling verification except when building diagnostic tools like the certificate inspector project.

## Exercises

### Exercise 1: HTTPS GET (implemented in 15-https-client.rs)
Connect to `example.com:443` over TLS, send an HTTP GET request, print the response.

### Exercise 2: Print TLS details
After the handshake, print:
- TLS protocol version (should be TLS 1.3)
- Negotiated cipher suite
- Server certificate subject name
- Certificate chain (list each cert's subject and issuer)

### Exercise 3: Try different sites
Connect to `google.com`, `github.com`, `cloudflare.com`. Compare the cipher suites and certificate chains. Some use ECDSA certificates, some use RSA. Some have 2-cert chains, some have 3.

### Exercise 4: Certificate pinning
Hardcode the expected SHA-256 fingerprint of `example.com`'s certificate. After the handshake, compute the fingerprint of the received certificate and compare. If they don't match, abort. This is certificate pinning — extra security beyond CA validation.

### Exercise 5: Connect without trusting the CA
Create a `ClientConfig` with an empty root store. Try connecting to `example.com`. The handshake should fail with a certificate verification error. This proves that the CA trust chain is enforced.
