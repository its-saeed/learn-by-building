// Lesson 8: Async I/O Foundations
//
// Explore blocking, non-blocking, and event-driven I/O patterns.
// Run with: cargo run -p async-lessons --bin 8-async-io -- <command>
//
// Commands:
//   blocking       Demo a blocking TCP read (std::net)
//   nonblocking    Demo a non-blocking TCP read with WouldBlock
//   mio-echo       Build a simple echo server using mio event loop
//   all            Run all demos in sequence
//
// NOTE: This lesson requires the `mio` crate. Add to async/Cargo.toml:
//   mio = { version = "1", features = ["os-poll", "net"] }

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "async-io", about = "Async I/O foundations: blocking, non-blocking, event-driven")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Demo: blocking TCP read (thread freezes until data arrives)
    Blocking,
    /// Demo: non-blocking TCP read with WouldBlock retry loop
    Nonblocking,
    /// Demo: mio-based echo server (event-driven, no busy-wait)
    MioEcho,
    /// Run all demos in sequence
    All,
}

// ─── Step markers ──────────────────────────────────────────

fn step(n: usize, msg: &str) {
    println!("\n  ▸ Step {n}: {msg}");
}

// ─── Pre-built: helper to spawn a client that sends data after a delay ────

/// Spawns a background thread that connects to `addr`, waits `delay`,
/// sends `message`, then closes the connection.
fn spawn_delayed_client(addr: &str, delay: Duration, message: &str) {
    let addr = addr.to_string();
    let message = message.to_string();
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        match TcpStream::connect(&addr) {
            Ok(mut stream) => {
                let _ = stream.write_all(message.as_bytes());
                println!("    [client] sent: {message}");
            }
            Err(e) => eprintln!("    [client] connect failed: {e}"),
        }
    });
}

// ─── Pre-built: blocking demo ──────────────────────────────

/// Demonstrates blocking I/O. The thread freezes on `read()` until
/// the client sends data.
fn demo_blocking() {
    println!("=== Blocking TCP Read ===");
    println!("  The main thread will freeze until data arrives.\n");

    step(1, "Bind a TCP listener on 127.0.0.1:0 (random port)");
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
    let addr = listener.local_addr().unwrap();
    println!("    Listening on {addr}");

    step(2, "Spawn a client that sends data after 1 second");
    spawn_delayed_client(&addr.to_string(), Duration::from_secs(1), "hello from blocking client");

    step(3, "Accept a connection (blocks until client connects)");
    let (mut stream, client_addr) = listener.accept().expect("accept failed");
    println!("    Accepted connection from {client_addr}");

    step(4, "Read from stream (blocks until data arrives)");
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).expect("read failed");
    let msg = String::from_utf8_lossy(&buf[..n]);
    println!("    Received {n} bytes: \"{msg}\"");

    println!("\n  Takeaway: the thread was frozen during the 1s wait.");
    println!("  With 10,000 connections, you'd need 10,000 threads.\n");
}

// ─── TODO: non-blocking demo ──────────────────────────────

/// Demonstrates non-blocking I/O with WouldBlock.
///
/// TODO: Implement this function.
///   1. Bind a TcpListener on 127.0.0.1:0
///   2. Spawn a delayed client (use spawn_delayed_client, 2s delay)
///   3. Set the listener to non-blocking: listener.set_nonblocking(true)
///   4. Accept in a loop:
///      - On Ok((stream, addr)) → got a connection, proceed
///      - On Err with WouldBlock → print "waiting..." and sleep 200ms
///      - On other Err → panic
///   5. Set the accepted stream to non-blocking
///   6. Read in a loop:
///      - On Ok(0) → client disconnected
///      - On Ok(n) → print the data, break
///      - On Err with WouldBlock → print "no data yet" and sleep 200ms
///      - On other Err → panic
///   7. Print how many WouldBlock retries happened for accept and read
///
/// This shows the busy-wait problem: you burn CPU checking in a loop.
fn demo_nonblocking() {
    println!("=== Non-blocking TCP Read ===");
    println!("  We poll in a loop, getting WouldBlock until data arrives.\n");

    todo!("Implement demo_nonblocking — see steps above")
}

// ─── TODO: mio echo server ─────────────────────────────────

/// Builds a multi-client echo server using mio's event loop.
///
/// TODO: Implement this function.
///   1. Create a mio::Poll and mio::Events (capacity 128)
///   2. Create a mio::net::TcpListener on 127.0.0.1:0
///   3. Register the listener with Token(0) for READABLE interest
///   4. Print the listening address
///   5. Spawn 3 delayed clients (delays: 500ms, 1s, 1.5s) that each
///      send a message and disconnect
///   6. Keep a HashMap<mio::Token, mio::net::TcpStream> for connections
///   7. Use a `next_token` counter starting at 1
///   8. Event loop (with a timeout of 3 seconds on poll to auto-exit):
///      - Token(0) event → accept connection, register with Token(next_token),
///        store in map, increment next_token
///      - Other token → read from the connection:
///        - Ok(0) → client disconnected, deregister, remove from map
///        - Ok(n) → echo back (write the same bytes), print what was echoed
///        - WouldBlock → continue (spurious wakeup)
///        - Other error → remove connection
///   9. Exit after the poll timeout fires with 0 events (all clients done)
///
/// This is the pattern that tokio's I/O driver uses internally.
fn demo_mio_echo() {
    println!("=== mio Echo Server ===");
    println!("  Event-driven: we sleep until the OS says a socket is ready.\n");

    todo!("Implement demo_mio_echo — see steps above")
}

// ─── Main ───────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Blocking => demo_blocking(),
        Command::Nonblocking => demo_nonblocking(),
        Command::MioEcho => demo_mio_echo(),
        Command::All => {
            demo_blocking();
            println!("\n{}\n", "─".repeat(60));
            demo_nonblocking();
            println!("\n{}\n", "─".repeat(60));
            demo_mio_echo();
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that set_nonblocking produces WouldBlock on an empty socket.
    #[test]
    fn test_nonblocking_would_block() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // Connect a client so accept succeeds
        let _client = TcpStream::connect(addr).unwrap();
        let (mut stream, _) = listener.accept().unwrap();

        // Set non-blocking — reading an empty socket should give WouldBlock
        stream.set_nonblocking(true).unwrap();
        let mut buf = [0u8; 64];
        let result = stream.read(&mut buf);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            io::ErrorKind::WouldBlock,
            "expected WouldBlock on empty non-blocking socket"
        );
    }

    /// Test that we can read data from a non-blocking socket when data is present.
    #[test]
    fn test_nonblocking_read_after_write() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let mut client = TcpStream::connect(addr).unwrap();
        let (mut server_stream, _) = listener.accept().unwrap();

        // Write from client
        client.write_all(b"ping").unwrap();

        // Give the OS a moment to deliver the data
        std::thread::sleep(Duration::from_millis(50));

        // Read from server in non-blocking mode
        server_stream.set_nonblocking(true).unwrap();
        let mut buf = [0u8; 64];
        let n = server_stream.read(&mut buf).unwrap();

        assert_eq!(&buf[..n], b"ping");
    }

    /// Test that a blocking read succeeds when data is available.
    #[test]
    fn test_blocking_read() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn a client that sends data immediately
        let handle = std::thread::spawn(move || {
            let mut client = TcpStream::connect(addr).unwrap();
            client.write_all(b"hello").unwrap();
        });

        let (mut stream, _) = listener.accept().unwrap();
        // Set a read timeout so this test doesn't hang forever on failure
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();

        let mut buf = [0u8; 64];
        let n = stream.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"hello");

        handle.join().unwrap();
    }
}
