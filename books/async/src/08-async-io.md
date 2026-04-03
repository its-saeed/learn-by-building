# Lesson 8: Async I/O Foundations

## Real-life analogy: fishing with multiple rods

Imagine you have 10 fishing rods and a lake full of fish.

**Blocking I/O** = hold one rod at a time, stare at it:

```
You:   [hold rod 1........waiting........waiting........FISH!]
       [hold rod 2........waiting........waiting........FISH!]
       [hold rod 3........waiting............................]

Total: you can only fish one rod at a time.
       9 rods sit idle while you stare at one.
```

**Non-blocking I/O** = plant all 10 rods, walk between them checking each:

```
You:   [check rod 1: nothing]
       [check rod 2: nothing]
       [check rod 3: FISH! → reel it in]
       [check rod 4: nothing]
       [check rod 5: nothing]
       [check rod 1: nothing]  ← loop back
       [check rod 2: FISH! → reel it in]
       ...

Total: you catch fish faster, but you burn energy walking
       back and forth even when nothing is biting (busy-wait).
```

**Event-driven I/O** = attach a bell to each rod, sit and wait for a bell:

```
You:   [sitting..............................RING! rod 3]
       [reel in rod 3]
       [sitting..........RING! rod 7, rod 1]
       [reel in rod 7, reel in rod 1]
       ...

Total: zero wasted effort. You only act when something
       is actually ready. This is kqueue / epoll.
```

The bell system is exactly how modern async I/O works:
1. Plant rods (open sockets, register with kqueue/epoll)
2. Sit and wait (call `kqueue_wait` / `epoll_wait`)
3. Bell rings (OS says "fd 7 is readable")
4. Reel in (read the data)

## How the OS tells you a socket is ready

When your program calls `read()` on a TCP socket, the kernel checks if any
data has arrived in the socket's receive buffer. If not, blocking `read()`
puts your thread to sleep. In async, you cannot afford to sleep — you need
the OS to *notify* you instead.

### kqueue (macOS) / epoll (Linux)

These are kernel APIs for event notification:

1. **Create** an event queue: `kqueue()` or `epoll_create()`
2. **Register** interest: "tell me when fd 5 is readable"
3. **Wait**: block until ANY registered fd has an event
4. **Process**: handle the ready fds
5. **Loop** back to step 3

```c
// Pseudocode (kqueue)
int kq = kqueue();
register(kq, socket_fd, EVFILT_READ);  // "notify me when readable"

loop {
    int n = kevent(kq, NULL, 0, events, MAX, NULL);  // wait
    for (int i = 0; i < n; i++) {
        int ready_fd = events[i].ident;
        // ready_fd has data — read without blocking
    }
}
```

One `kevent()` call can watch **thousands** of file descriptors simultaneously.
This is why nginx and tokio can handle 100K connections on a single thread.

## The three syscall patterns

### Pattern 1: Blocking read

```
Thread          Kernel
  │                │
  │── read(fd) ───>│
  │   (blocked)    │  ...waiting for data...
  │   (blocked)    │  ...still waiting...
  │<── data ───────│
  │                │
```

Thread is frozen. Cannot do anything else. One thread per connection.

### Pattern 2: Non-blocking read (poll loop)

```
Thread              Kernel
  │                    │
  │── read(fd) ───────>│
  │<── WouldBlock ─────│  (no data yet)
  │                    │
  │── read(fd) ───────>│
  │<── WouldBlock ─────│  (still no data)
  │                    │
  │── read(fd) ───────>│
  │<── 42 bytes ───────│  (data arrived!)
  │                    │
```

Thread is not frozen, but it wastes CPU spinning in a loop.
This is the "walking between fishing rods" approach.

### Pattern 3: Event notification (kqueue/epoll)

```
Thread              Kernel (kqueue)
  │                    │
  │── register(fd) ───>│  "watch fd for readability"
  │<── ok ─────────────│
  │                    │
  │── wait() ─────────>│
  │   (sleeping)       │  ...kernel watches all fds...
  │   (sleeping)       │  ...data arrives on fd...
  │<── [fd ready] ─────│  "fd has data"
  │                    │
  │── read(fd) ───────>│
  │<── 42 bytes ───────│  (guaranteed not to block)
  │                    │
```

Thread sleeps efficiently (no CPU usage). Kernel wakes it only
when something is ready. One thread watches thousands of fds.

### Timeline comparison

```
Time ──────────────────────────────────────────────────>

Blocking (3 sockets, 3 threads):
  Thread 1: ████████████████░░░░░░░░░░░  (blocked on fd 1)
  Thread 2: ░░░░░░░░████████████████░░░  (blocked on fd 2)
  Thread 3: ░░░░░░░░░░░░░░░░████████░░░  (blocked on fd 3)
  Cost: 3 threads, 24 MB stack memory

Non-blocking (3 sockets, 1 thread):
  Thread 1: ○○○○○●○○○○○●○○●○○○○○○○○○○○  (● = data, ○ = WouldBlock)
  Cost: 1 thread, but 100% CPU usage spinning

Event-driven (3 sockets, 1 thread):
  Thread 1: ___________●____●__●________  (● = event, _ = sleeping)
  Cost: 1 thread, near-zero CPU when idle
```

## Non-blocking sockets in Rust

Standard library sockets are blocking by default. You flip them to
non-blocking mode with one call:

```rust
use std::net::TcpStream;
use std::io::{self, Read};

let stream = TcpStream::connect("127.0.0.1:8080")?;
stream.set_nonblocking(true)?;  // ← the magic switch

let mut buf = [0u8; 1024];
match stream.read(&mut buf) {
    Ok(n) => println!("got {n} bytes"),
    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        println!("no data yet — try again later");
    }
    Err(e) => eprintln!("real error: {e}"),
}
```

`WouldBlock` is not a failure — it means "nothing to read right now."
The key insight: in non-blocking mode, you get to *choose* when to retry
instead of having the OS freeze your thread.

## kqueue/epoll explained

The event loop pattern is the same on every OS, just different syscalls:

```
┌─────────────────────────────────────────────────────┐
│                    Event Loop                        │
│                                                      │
│   ┌──────────────┐     ┌──────────────┐             │
│   │  Register fd │────>│   kqueue /   │             │
│   │  + interest  │     │   epoll      │             │
│   └──────────────┘     │  (kernel)    │             │
│                        └──────┬───────┘             │
│                               │                      │
│                        ┌──────▼───────┐             │
│                        │    wait()    │             │
│                        │  (sleeps)   │             │
│                        └──────┬───────┘             │
│                               │ wakes up            │
│                        ┌──────▼───────┐             │
│                        │ ready fds:   │             │
│                        │ [3, 7, 12]   │             │
│                        └──────┬───────┘             │
│                               │                      │
│                        ┌──────▼───────┐             │
│                        │ process each │─── loop ──┐ │
│                        │   ready fd   │           │ │
│                        └──────────────┘           │ │
│                               ▲                    │ │
│                               └────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

Steps:
1. **Create**: `kqueue()` returns a file descriptor for the event queue itself
2. **Register**: `kevent(kq, &changes, ...)` — add fds you care about
3. **Wait**: `kevent(kq, NULL, 0, &events, max, timeout)` — blocks until ready
4. **Process**: iterate over returned events, read/write the ready fds
5. **Repeat**: go back to step 3

## The mio crate

`mio` (Metal I/O) is a thin, cross-platform wrapper around kqueue/epoll/IOCP.
Tokio is built on top of mio. The core types:

| Type | Purpose |
|------|---------|
| `Poll` | Owns the kqueue/epoll fd. You call `poll.poll()` to wait. |
| `Events` | Buffer that `poll()` fills with ready events. |
| `Token(usize)` | Your label for each fd. When an event fires, you get the token back. |
| `Interest` | What you care about: `READABLE`, `WRITABLE`, or both. |
| `Registry` | Obtained from `poll.registry()`. Used to register/deregister fds. |

```rust
use mio::{Poll, Events, Token, Interest};
use mio::net::TcpListener;

let mut poll = Poll::new()?;
let mut events = Events::with_capacity(128);

let addr = "127.0.0.1:9000".parse()?;
let mut listener = TcpListener::bind(addr)?;

// Register: "tell me when listener has a new connection"
poll.registry().register(&mut listener, Token(0), Interest::READABLE)?;

loop {
    // Wait: sleep until something is ready
    poll.poll(&mut events, None)?;

    for event in events.iter() {
        match event.token() {
            Token(0) => {
                // Listener is readable → accept new connection
                let (mut conn, addr) = listener.accept()?;
                println!("new connection from {addr}");

                // Register the new connection too
                poll.registry().register(
                    &mut conn,
                    Token(1),
                    Interest::READABLE,
                )?;
            }
            Token(1) => {
                // Connection is readable → read data
            }
            _ => unreachable!(),
        }
    }
}
```

## How this connects to the async executor

The bridge from OS events to futures looks like this:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│    Kernel     │     │   Reactor    │     │   Executor   │
│  (kqueue/     │     │  (mio Poll   │     │  (task queue  │
│   epoll)      │     │   loop)      │     │   + polling)  │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                     │                     │
       │  fd 7 readable      │                     │
       │────────────────────>│                     │
       │                     │  waker.wake()       │
       │                     │  for task on fd 7   │
       │                     │────────────────────>│
       │                     │                     │  poll task's
       │                     │                     │  future
       │                     │                     │──┐
       │                     │                     │  │ Future::poll()
       │                     │                     │<─┘ → Ready(data)
```

1. Future calls `read()` on a non-blocking socket → gets `WouldBlock`
2. Future registers the fd with the reactor and stores the `Waker`
3. Future returns `Poll::Pending`
4. Reactor's mio poll loop eventually gets an event for that fd
5. Reactor calls `waker.wake()` for the associated task
6. Executor re-polls the future
7. This time, `read()` succeeds → future returns `Poll::Ready(data)`

This is the complete chain. Lesson 9 builds the reactor. This lesson gives
you the foundation: raw I/O primitives that the reactor wraps.

## Exercises

### Exercise 1: Raw non-blocking socket
Create a TCP listener. Accept a connection with `set_nonblocking(true)`.
Try to read in a loop — print each `WouldBlock` and sleep 100ms between
retries. When data arrives, print it and exit. Use only `std::net`, no mio.

### Exercise 2: kqueue/mio event loop
Replace the busy-wait loop from Exercise 1 with `mio::Poll`. Register the
accepted connection for `READABLE` interest. Call `poll.poll()` to sleep
until data arrives. Read and print. Compare CPU usage with Exercise 1.

### Exercise 3: mio TCP echo server
Build a multi-client echo server using mio. The listener gets `Token(0)`.
Each accepted connection gets `Token(next_id)`. Store connections in a
`HashMap<Token, TcpStream>`. On a readable event, read data and write it
back. Handle client disconnections by deregistering and removing from the map.

### Exercise 4: Connect reactor to waker
Extend Exercise 2: instead of reading directly in the event handler, store
a `Waker` when you register the fd. When the event fires, call
`waker.wake()` instead of reading. In a separate "executor" loop, poll
the future which then does the actual read. This is the reactor pattern
that Lesson 9 will build in full.
