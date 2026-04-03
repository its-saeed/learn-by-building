# Lesson 0: Why Async?

## The problem: one thread per connection

The traditional server model: accept a connection, spawn a thread to handle it.

```rust
loop {
    let stream = listener.accept();
    std::thread::spawn(|| handle(stream));
}
```

This works until you have thousands of connections. Each thread costs:
- **Stack memory**: 2-8 MB per thread (default on Linux/macOS)
- **Context switching**: OS must save/restore registers, flush caches
- **Scheduling overhead**: OS scheduler wasn't designed for 10K+ threads

10,000 threads × 8 MB = **80 GB** of stack space. That's the **C10K problem** — handling 10,000 concurrent connections.

## The solution: event-driven I/O

Instead of blocking a thread per connection, use **one thread** (or a few) that watches many connections simultaneously:

1. Register interest: "tell me when socket 5 has data to read"
2. Wait: block on a single syscall (`epoll`/`kqueue`) until ANY socket is ready
3. Handle: process only the sockets that are ready
4. Repeat

No wasted threads, no wasted memory. One thread can handle 100K+ connections.

## Where async fits

Writing event-driven code by hand is painful — callback hell, manual state machines. Rust's `async`/`.await` gives you event-driven performance with sequential-looking code:

```rust
// Looks sequential, but doesn't block a thread
async fn handle(stream: TcpStream) {
    let data = stream.read().await;  // yields, doesn't block
    stream.write(data).await;        // yields, doesn't block
}
```

The compiler transforms this into a state machine. The runtime (tokio) handles the event loop. You write simple code, get scalable performance.

## The cost of async

Async isn't free:
- **Complexity**: Pin, lifetimes across await points, cancellation safety
- **Debugging**: stack traces are less readable
- **Ecosystem split**: async and sync code don't mix easily

Use async when you have many concurrent I/O operations. Don't use it for CPU-bound work or simple scripts.

## Exercises

### Exercise 1: Thread overhead benchmark
Spawn 10,000 threads that each sleep for 1 second. Measure total memory usage and wall time. Then do the same with `tokio::spawn` and 10,000 async tasks. Compare.

### Exercise 2: Max threads
Keep spawning threads until the OS refuses. How many can you create? What error do you get?

### Exercise 3: Blocking vs non-blocking
Open a TCP connection with `std::net::TcpStream` — it blocks. Open one with `tokio::net::TcpStream` — it doesn't. What does "blocking" actually mean at the syscall level? Use `strace`/`dtruss` to see the difference.
