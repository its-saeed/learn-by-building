// Project 1: Mini Executor + TCP Echo Server
//
// Build a single-threaded async runtime from scratch and run a real TCP echo
// server on it. No tokio, no external async runtime — you implement every layer:
// executor, reactor, async TCP types.
//
// Combines: Lessons 1-8 (futures, state machines, wakers, tasks, executor,
// pinning, combinators, async I/O, reactor).
//
// Run with: cargo run -p async-lessons --bin p1-echo-server -- <command>
//
// Commands:
//   run         Start the echo server on 127.0.0.1:8080
//   test        Run a quick self-test (spawn server, connect, check echo)
//
// NOTE: add `mio = { version = "1", features = ["os-poll", "net"] }` to Cargo.toml

use clap::{Parser, Subcommand};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

// ============================================================
// Reactor — bridges OS events (kqueue/epoll via mio) to wakers
// ============================================================

/// Unique identifier for a registered I/O source.
/// Wraps a usize that maps to a mio::Token internally.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct Token(usize);

/// The Reactor owns the mio::Poll and maps tokens to wakers.
/// When an I/O event fires, it wakes the corresponding task.
///
/// TODO: Implement the Reactor.
///   - Store a mio::Poll, a HashMap<Token, Waker>, and a token counter
///   - register(source, interest) -> Token: register a socket with mio
///   - set_waker(token, waker): store/update the waker for a token
///   - deregister(token): remove a socket and its waker
///   - wait(): call poll.poll(), then wake() each token that has an event
struct Reactor {
    // poll: mio::Poll,
    wakers: HashMap<Token, Waker>,
    next_token: usize,
}

impl Reactor {
    /// Create a new Reactor with a fresh mio::Poll.
    fn new() -> io::Result<Self> {
        todo!("Create mio::Poll, return Reactor")
    }

    /// Register an I/O source (e.g. TcpListener, TcpStream) with the reactor.
    /// Returns a Token that identifies this source in future events.
    ///
    /// TODO:
    ///   1. Assign the next token (self.next_token), increment counter
    ///   2. Call self.poll.registry().register(source, mio_token, interest)
    ///   3. Return the Token
    fn register(
        &mut self,
        // source: &mut impl mio::event::Source,
        // interest: mio::Interest,
    ) -> Token {
        todo!("Register source with mio::Poll")
    }

    /// Store or update the waker for a given token.
    /// Called by futures in their poll() when they return Pending.
    fn set_waker(&mut self, token: Token, waker: Waker) {
        self.wakers.insert(token, waker);
    }

    /// Remove a source from the reactor.
    ///
    /// TODO:
    ///   1. Deregister the source from mio::Poll
    ///   2. Remove the waker from self.wakers
    fn deregister(
        &mut self,
        _token: Token,
        // source: &mut impl mio::event::Source,
    ) {
        todo!("Deregister source from mio::Poll, remove waker")
    }

    /// Block until at least one I/O event fires, then wake the corresponding tasks.
    ///
    /// TODO:
    ///   1. Call self.poll.poll(&mut events, timeout)
    ///   2. For each event, look up the waker by token
    ///   3. Call waker.wake_by_ref() to push the task back into the executor queue
    fn wait(&mut self, _timeout: Option<std::time::Duration>) {
        todo!("Poll for events and wake tasks")
    }
}

// ============================================================
// Task + Executor — a simple single-threaded task scheduler
// ============================================================

/// A spawned future wrapped with executor metadata.
/// The executor polls tasks; the waker re-queues them.
struct Task {
    /// The future this task drives. Pinned + boxed for type erasure.
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    /// Handle to the shared task queue so the waker can re-enqueue this task.
    queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
}

impl Wake for Task {
    /// Called by the reactor when I/O is ready. Pushes this task back into the
    /// executor's queue so it gets polled again.
    fn wake(self: Arc<Self>) {
        self.queue.lock().unwrap().push_back(self.clone());
    }
}

/// The shared task queue. All spawned tasks live here.
/// In a real runtime this would be thread-local or behind a more
/// sophisticated scheduler.
fn task_queue() -> Arc<Mutex<VecDeque<Arc<Task>>>> {
    // Use a thread-local or lazy_static in your implementation.
    // For the skeleton we just create a fresh one (replace this).
    todo!("Return a shared task queue (e.g. thread_local! or static)")
}

/// Spawn a future as a new task on the executor.
///
/// TODO:
///   1. Wrap the future in a Task (Arc<Task>)
///   2. Push it into the task queue
///   3. (Optional) return a JoinHandle
pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    let queue = task_queue();
    let task = Arc::new(Task {
        future: Mutex::new(Box::pin(future)),
        queue: queue.clone(),
    });
    queue.lock().unwrap().push_back(task);
}

/// Run a future to completion on the current thread.
///
/// This is the entry point of your runtime. It:
///   1. Pins the main future
///   2. Polls it (and all spawned tasks) in a loop
///   3. When nothing is ready, calls reactor.wait() to block for I/O events
///   4. Returns when the main future resolves
///
/// TODO:
///   1. Create a waker for the main future (thread::current().unpark())
///   2. Loop:
///      a. Poll the main future — if Ready, return
///      b. Drain the task queue and poll each task
///      c. If no progress was made, call reactor.wait()
pub fn block_on<F: Future>(future: F) -> F::Output {
    todo!("Implement the main executor loop")
}

// ============================================================
// Async TCP types — wrappers around mio sockets
// ============================================================

/// Async TCP listener. Wraps mio::net::TcpListener + a reactor Token.
///
/// TODO:
///   - Store the mio TcpListener and its Token
///   - bind(addr) creates the listener, sets non-blocking, registers with reactor
///   - accept() returns a future that yields (TcpStream, SocketAddr)
pub struct TcpListener {
    // inner: mio::net::TcpListener,
    token: Token,
}

impl TcpListener {
    /// Bind to an address and register with the reactor.
    ///
    /// TODO:
    ///   1. Create a mio::net::TcpListener::bind(addr)
    ///   2. Register it with the reactor for READABLE interest
    ///   3. Return TcpListener { inner, token }
    pub async fn bind(_addr: &str) -> Self {
        todo!("Bind TCP listener and register with reactor")
    }

    /// Accept the next incoming connection.
    ///
    /// TODO: return a future that:
    ///   1. Tries self.inner.accept()
    ///   2. If Ok((stream, addr)) → wrap stream in TcpStream, return Ready
    ///   3. If Err(WouldBlock) → store waker with reactor, return Pending
    ///   4. Other errors → panic or propagate
    pub async fn accept(&self) -> TcpStream {
        todo!("Accept a connection (future that registers with reactor)")
    }
}

/// Async TCP stream. Wraps mio::net::TcpStream + a reactor Token.
///
/// TODO:
///   - Store the mio TcpStream and its Token
///   - read(&mut buf) returns a future that yields usize (bytes read)
///   - write_all(data) returns a future that writes everything
pub struct TcpStream {
    // inner: mio::net::TcpStream,
    token: Token,
}

impl TcpStream {
    /// Read data into the buffer. Returns number of bytes read (0 = EOF).
    ///
    /// TODO: return a future that:
    ///   1. Tries self.inner.read(buf) (using std::io::Read)
    ///   2. If Ok(n) → Ready(n)
    ///   3. If Err(WouldBlock) → store waker, Pending
    pub async fn read(&self, _buf: &mut [u8]) -> usize {
        todo!("Async read from TcpStream")
    }

    /// Write all data to the stream.
    ///
    /// TODO: return a future that:
    ///   1. Tracks how many bytes have been written so far
    ///   2. Tries self.inner.write(&data[written..])
    ///   3. If all written → Ready(())
    ///   4. If WouldBlock → register for WRITABLE, Pending
    pub async fn write_all(&self, _data: &[u8]) {
        todo!("Async write_all to TcpStream")
    }
}

// ============================================================
// Echo handler — the application logic
// ============================================================

/// Handle a single client: read data, echo it back, repeat until EOF.
async fn handle_client(stream: TcpStream) {
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf).await;
        if n == 0 {
            println!("  client disconnected");
            return;
        }
        let msg = String::from_utf8_lossy(&buf[..n]);
        print!("  echo: {}", msg);
        stream.write_all(&buf[..n]).await;
    }
}

// ============================================================
// Goal code — uncomment once you've implemented everything above
// ============================================================

// fn run_echo_server() {
//     println!("Echo server listening on 127.0.0.1:8080");
//     println!("Test with: nc 127.0.0.1 8080");
//     println!();
//
//     block_on(async {
//         let listener = TcpListener::bind("127.0.0.1:8080").await;
//         loop {
//             let stream = listener.accept().await;
//             println!("  new client connected");
//             spawn(handle_client(stream));
//         }
//     });
// }

// ============================================================
// CLI
// ============================================================

#[derive(Parser)]
#[command(
    name = "p1-echo-server",
    about = "Project 1: Build an async runtime from scratch and run a TCP echo server"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the echo server on 127.0.0.1:8080
    Run,
    /// Run a quick self-test: spawn server, connect, check echo
    Test,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Run => {
            println!("=== Project 1: TCP Echo Server ===");
            println!();
            println!("You need to implement the runtime first!");
            println!("Uncomment run_echo_server() in main() when ready.");
            println!();
            println!("Implementation order:");
            println!("  1. Reactor::new(), register(), wait()");
            println!("  2. block_on() — poll main future + drain task queue");
            println!("  3. TcpListener::bind(), accept()");
            println!("  4. TcpStream::read(), write_all()");
            println!("  5. spawn() — wire task waker to queue");
            println!("  6. Wire reactor.wait() into the block_on loop");
            println!();
            println!("Then uncomment run_echo_server() and test with:");
            println!("  nc 127.0.0.1 8080");

            // Uncomment when implemented:
            // run_echo_server();
        }
        Command::Test => {
            println!("=== Self-test ===");
            println!();
            println!("This will start the server, connect a client, send");
            println!("\"hello\", and verify the echo. Not yet implemented.");
            println!();
            println!("Implement run_self_test() once your runtime works.");

            // Uncomment when implemented:
            // run_self_test();
        }
    }
}

// fn run_self_test() {
//     use std::io::{Read, Write};
//     use std::net::TcpStream as StdTcpStream;
//     use std::time::Duration;
//
//     // Spawn the server in a background thread
//     std::thread::spawn(|| {
//         block_on(async {
//             let listener = TcpListener::bind("127.0.0.1:18080").await;
//             loop {
//                 let stream = listener.accept().await;
//                 spawn(handle_client(stream));
//             }
//         });
//     });
//
//     // Give the server a moment to start
//     std::thread::sleep(Duration::from_millis(100));
//
//     // Connect and test echo
//     let mut client = StdTcpStream::connect("127.0.0.1:18080").unwrap();
//     client.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
//     client.write_all(b"hello\n").unwrap();
//
//     let mut buf = [0u8; 64];
//     let n = client.read(&mut buf).unwrap();
//     assert_eq!(&buf[..n], b"hello\n");
//     println!("Self-test PASSED: sent \"hello\", received \"hello\"");
// }

// ============================================================
// Tests — run with: cargo test -p async-lessons --bin p1-echo-server
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that the Task wake mechanism correctly re-enqueues a task.
    #[test]
    fn test_task_wake_requeues() {
        let queue: Arc<Mutex<VecDeque<Arc<Task>>>> =
            Arc::new(Mutex::new(VecDeque::new()));

        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(async {})),
            queue: queue.clone(),
        });

        // Queue should be empty
        assert_eq!(queue.lock().unwrap().len(), 0);

        // Wake the task — it should push itself into the queue
        task.wake();

        assert_eq!(queue.lock().unwrap().len(), 1);
    }

    /// Test that the Token type works as a HashMap key.
    #[test]
    fn test_token_as_map_key() {
        let mut map: HashMap<Token, Waker> = HashMap::new();

        // We can't easily create a real Waker without an executor, so just
        // verify the Token hashing and equality work.
        let t1 = Token(0);
        let t2 = Token(1);
        let t3 = Token(0);

        assert_eq!(t1, t3);
        assert_ne!(t1, t2);

        // Verify it compiles as a key (the type system checks the rest).
        // Using a noop waker to actually insert.
        let noop_waker = Arc::new(NoopWaker).into();
        map.insert(t1, noop_waker);
        assert!(map.contains_key(&t3));
        assert!(!map.contains_key(&t2));
    }

    /// Minimal waker for testing. Does nothing on wake.
    struct NoopWaker;

    impl Wake for NoopWaker {
        fn wake(self: Arc<Self>) {}
    }
}
