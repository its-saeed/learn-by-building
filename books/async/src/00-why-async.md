# Lesson 0: Why Async?

## Real-life analogy: the restaurant

Imagine a restaurant with 100 tables.

**Thread-per-connection model** = one waiter per table:

```
Table 1 → Waiter 1 (stands at table, waits for customer to decide)
Table 2 → Waiter 2 (stands at table, waits for food from kitchen)
Table 3 → Waiter 3 (stands at table, waits for customer to finish)
...
Table 100 → Waiter 100
```

Each waiter does nothing most of the time — they're **blocked** waiting. You need 100 waiters (expensive), and they're mostly idle. If 200 guests show up, you can't serve them.

**Event-driven model** = one waiter, many tables:

```
Waiter checks Table 1 → "still reading menu, skip"
Waiter checks Table 2 → "food is ready!" → serves it
Waiter checks Table 3 → "wants to order!" → takes order, sends to kitchen
Waiter checks Table 4 → "still eating, skip"
...
```

One waiter handles all tables by only doing work when a table needs something. The waiter never stands idle — they keep circling. This is **event-driven I/O**.

The buzzer system at a fast-food restaurant is even more accurate:
1. You order (register interest)
2. You sit down and do other things (non-blocking)
3. Buzzer vibrates (event notification — kqueue/epoll)
4. You pick up your food (handle the ready event)

## The problem: one thread per connection

The traditional server model:

```rust
loop {
    let stream = listener.accept();
    std::thread::spawn(|| handle(stream));  // one thread per client
}
```

Each thread costs real resources:

```
┌────────────────────────────────────────────────────┐
│              Thread Memory Layout                  │
│                                                    │
│  ┌──────────────┐  Each thread gets its own stack  │
│  │ Stack: 8 MB  │  allocated by the OS.            │
│  │              │  Most of it is never used —      │
│  │  (mostly     │  the thread is just waiting      │
│  │   empty,     │  on read() or write().           │
│  │   waiting)   │                                  │
│  │              │                                  │
│  └──────────────┘                                  │
│                                                    │
│  10,000 threads × 8 MB = 80 GB virtual memory      │
│  (actual RSS is lower, but overhead is still real) │
└────────────────────────────────────────────────────┘
```

### See it yourself

Check your system's default thread stack size:

```sh
# macOS
ulimit -s          # prints stack size in KB (usually 8192 = 8 MB)

# Linux
ulimit -s          # usually 8192 KB
cat /proc/sys/kernel/threads-max   # max threads the kernel allows
```

Check how many threads a process is using:

```sh
# macOS: count threads of a process
ps -M <pid> | wc -l

# Linux: count threads of a process
ls /proc/<pid>/task | wc -l

# Or for any process by name
ps -eLf | grep <process-name> | wc -l
```

### The C10K problem

In 1999, Dan Kegel asked: "how do you handle 10,000 concurrent connections?" With one thread per connection, you can't — you hit OS limits on memory, thread count, and context switching overhead.

```
Connections     Threads     Memory (8MB stack)    Context switches/sec
──────────────────────────────────────────────────────────────────────
100             100         800 MB                 ~10,000
1,000           1,000       8 GB                   ~100,000
10,000          10,000      80 GB                  ~1,000,000 (OS melts)
100,000         ???         impossible             impossible
```

Modern servers need to handle 100K-1M+ connections. Threads don't scale.

## The solution: event-driven I/O

Instead of blocking a thread per connection, use **one thread** that watches all connections:

```
┌──────────────────────────────────────────────────────────────┐
│                    Event-Driven Server                       │
│                                                              │
│  ┌─────────┐                                                 │
│  │ kqueue  │ ← register: "notify me when socket 5 is ready"  │
│  │ /epoll  │ ← register: "notify me when socket 8 is ready"  │
│  │         │ ← register: "notify me when socket 12 is ready" │
│  └────┬────┘                                                 │
│       │                                                      │
│       ▼  wait() — blocks until ANY socket is ready           │
│                                                              │
│  "Socket 8 is readable!"                                     │
│       │                                                      │
│       ▼  read from socket 8, process, respond                │
│       │                                                      │
│       ▼  back to wait()                                      │
│                                                              │
│  One thread. 100,000 connections. ~10 MB memory.             │
└──────────────────────────────────────────────────────────────┘
```

### See blocking in action

Run this in **Terminal 1** — a blocking TCP server:

```sh
python3 -c "
import socket, os

print('=== Blocking Server Demo ===')
print(f'PID: {os.getpid()}')
print()

s = socket.socket()
s.bind(('127.0.0.1', 9999))
s.listen()

# BLOCKING CALL #1: accept()
# The thread is FROZEN here. It can't do anything else.
# Open another terminal and run: echo hello | nc 127.0.0.1 9999
print('[1] Calling accept()... thread is BLOCKED, waiting for a connection')
print('    → Nothing else can happen on this thread until someone connects')
print('    → In another terminal run: echo hello | nc 127.0.0.1 9999')
print()
conn, addr = s.accept()
print(f'[2] accept() returned! Someone connected from {addr}')
print()

# BLOCKING CALL #2: recv()
# The thread is FROZEN again, waiting for data.
print('[3] Calling recv()... thread is BLOCKED again, waiting for data')
data = conn.recv(1024)
print(f'[4] recv() returned! Got: {data}')
print()

print('=== Takeaway ===')
print('Between [1] and [2], the thread did NOTHING. Completely idle.')
print('Between [3] and [4], same thing. Idle.')
print('With 10,000 clients, you need 10,000 threads all sitting idle like this.')
print('That is the problem async solves.')
"
```

In **Terminal 2**:
```sh
echo hello | nc 127.0.0.1 9999
```

Watch the output: the server prints `[1]`, then **freezes**. Nothing happens until you connect from Terminal 2. Then it prints `[2]`, `[3]`, and freezes again until data arrives. Each freeze = a wasted thread.

### See non-blocking behavior

```sh
python3 -c "
import socket

print('=== Non-Blocking Demo ===')
print()

s = socket.socket()
s.setblocking(False)  # <-- key difference: non-blocking mode

# Non-blocking connect returns IMMEDIATELY, even though the connection
# isn't established yet. Instead of freezing the thread, it raises an error.
print('[1] Calling connect() on a NON-BLOCKING socket...')
try:
    s.connect(('example.com', 80))
except BlockingIOError as e:
    print(f'[2] connect() returned INSTANTLY with: {e}')
    print('    → The thread was NOT frozen. It got WouldBlock immediately.')
    print('    → The connection is still in progress in the background.')
    print()

# Non-blocking recv also returns immediately if no data is ready.
print('[3] Calling recv() on a NON-BLOCKING socket (no data yet)...')
try:
    s.recv(1024)
except BlockingIOError as e:
    print(f'[4] recv() returned INSTANTLY with: {e}')
    print('    → No data yet, but the thread is FREE to do other work.')
    print()

print('=== Takeaway ===')
print('Non-blocking I/O never freezes the thread.')
print('Instead of waiting, you get WouldBlock and can go handle other connections.')
print('This is what async runtimes (tokio) do under the hood:')
print('  1. Try non-blocking read → WouldBlock')
print('  2. Register with kqueue/epoll: \"tell me when this socket is ready\"')
print('  3. Go handle other tasks')
print('  4. kqueue/epoll wakes you up → try read again → data is there')
"
```

### Blocking vs non-blocking: what happens at the OS level

When you call `read()` on a **blocking** socket:

```
Your code                           OS Kernel
    │                                 │
    ├── read(fd) ──────────────────►  │
    │   (your thread is FROZEN)       │  waiting for data...
    │   (can't do anything else)      │  still waiting...
    │                                 │  data arrives!
    │  ◄── returns data ──────────────┤
    │                                 │
```

When you call `read()` on a **non-blocking** socket:

```
Your code                           OS Kernel
    │                                 │
    ├── read(fd) ──────────────────►  │
    │  ◄── WouldBlock (instantly) ────┤  no data yet
    │                                 │
    │  (go do other work!)            │
    │                                 │
    ├── read(fd) ──────────────────►  │
    │  ◄── WouldBlock (instantly) ────┤  still no data
    │                                 │
    │  ... later, after kqueue says   │
    │      "fd is ready" ...          │
    │                                 │
    ├── read(fd) ──────────────────►  │
    │  ◄── returns data ──────────────┤  data was ready!
```

### See kqueue/epoll (the OS event system)

On **Linux**, you can trace the actual syscalls:

```sh
strace -e epoll_wait,epoll_ctl,accept,read python3 -c "
import socket
s = socket.socket()
s.bind(('127.0.0.1', 9999))
s.listen()
print('waiting for connection...')
s.accept()
"
# You'll see: epoll_create, epoll_ctl (register fd), epoll_wait (block until event)
```

On **macOS**, SIP restricts dtruss/dtrace. Instead, use `sample` to see where a process is blocked:

```sh
# While the blocking server above is waiting on accept():
sudo sample <pid> 1 2>&1 | grep -i 'accept\|kevent\|select'
# You'll see it stuck in accept() or kevent() — the thread is parked in the kernel
```

To check if syscall tracing is available on your Mac:
```sh
csrutil status
# If "System Integrity Protection status: enabled", dtruss is blocked.
# The Python demos above show the same concepts without needing dtruss.
```

## Where async fits

Writing event-driven code by hand is painful — you end up with callback spaghetti:

```rust
// Callback hell (event-driven without async)
socket.on_readable(|data| {
    process(data, |result| {
        socket.on_writable(|_| {
            socket.write(result, |_| {
                // deeply nested, hard to follow
            });
        });
    });
});
```

Rust's `async`/`.await` gives you event-driven performance with sequential-looking code:

```rust
// Same logic, but readable
async fn handle(stream: TcpStream) {
    let data = stream.read().await;   // yields to runtime, doesn't block thread
    let result = process(data);
    stream.write(result).await;       // yields to runtime, doesn't block thread
}
```

The compiler transforms this into a state machine (Lesson 2). The runtime (tokio) manages the event loop (Lesson 8). You write simple code, get scalable performance.

### The mental model

```
┌─────────────────────────────────────────────────────┐
│                What you write                       │
│                                                     │
│  async fn handle(stream: TcpStream) {               │
│      let data = stream.read().await;                │
│      stream.write(data).await;                      │
│  }                                                  │
└───────────────────┬─────────────────────────────────┘
                    │ compiler transforms
                    ▼
┌───────────────────────────────────────────────────────┐
│              What the compiler generates              │
│                                                       │
│  A state machine enum:                                │
│    State::Reading  → poll read, if not ready: Pending │
│    State::Writing  → poll write, if not ready: Pending│
│    State::Done     → return Ready(())                 │
└───────────────────┬───────────────────────────────────┘
                    │ runtime drives
                    ▼
┌─────────────────────────────────────────────────────┐
│              What the runtime does                  │
│                                                     │
│  loop {                                             │
│      events = kqueue.wait();                        │
│      for event in events {                          │
│          task = lookup(event.fd);                   │
│          task.poll(); // advance the state machine  │
│      }                                              │
│  }                                                  │
└─────────────────────────────────────────────────────┘
```

## The cost of async

Async isn't free. The trade-off:

```
                    Threads              Async
─────────────────────────────────────────────────────
Memory per task     2-8 MB (stack)       ~100 bytes (future struct)
Max connections     ~10K                 ~1M
Context switch      OS (expensive)       Userspace (cheap)
Code complexity     Simple               Pin, lifetimes, cancellation
Debugging           Good stack traces    Confusing stack traces
Ecosystem           Everything works     Need async versions of libs
CPU-bound work      Natural              Must use spawn_blocking
```

**Use async when**: many concurrent I/O operations (web servers, proxies, chat, databases)

**Don't use async when**: CPU-bound work, simple scripts, low concurrency, prototyping

## Exercises

### Exercise 1: Thread overhead benchmark

Spawn 10,000 threads that each `std::thread::sleep(Duration::from_secs(1))`. Measure:
- Wall time: `std::time::Instant::now()` before and after
- Peak memory: check with `ps` or `Activity Monitor` while running

Then do the same with 10,000 `tokio::spawn` tasks using `tokio::time::sleep`. Compare both.

Useful commands while the benchmark runs:
```sh
# macOS: check memory of your process
ps -o pid,rss,vsz,comm -p <pid>
# rss = actual memory used (KB), vsz = virtual memory (KB)

# Linux
cat /proc/<pid>/status | grep -E 'VmRSS|VmSize|Threads'
```

### Exercise 2: Max threads

Keep spawning `std::thread::spawn` in a loop until it fails. Print the count and the error.

```sh
# Check your system limits
ulimit -u    # max user processes
sysctl kern.num_taskthreads  # macOS max threads per process
```

### Exercise 3: Blocking vs non-blocking syscalls

Write two programs that connect to a TCP server and read data:
1. Using `std::net::TcpStream` (blocking)
2. Using `std::net::TcpStream` with `set_nonblocking(true)`

Trace the syscalls:
```sh
# macOS
sudo dtruss -f ./target/debug/my-binary 2>&1 | grep -E 'read|recvfrom|kevent'

# Linux
strace -e read,recvfrom,epoll_wait ./target/debug/my-binary
```

In the blocking version, you'll see `read()` that takes seconds to return.
In the non-blocking version, you'll see `read()` returning immediately with EAGAIN/EWOULDBLOCK.
