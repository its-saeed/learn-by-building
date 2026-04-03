# Project 1: Mini Executor + TCP Echo Server

> **Prerequisites**: Lessons 1-8 (futures, state machines, wakers, tasks,
> executor, pinning, combinators, async I/O, reactor). This project ties
> them all together into a working system.

## Overview

Build a **single-threaded async runtime from scratch** that runs a real TCP
echo server. No tokio. No external async runtime. You write every layer:
executor, reactor, async TCP types. Then you run real network I/O on it.

This is the moment everything clicks. You've built futures, wakers, an
executor, and a reactor in isolation. Now you combine them into one program
that handles multiple concurrent TCP clients on a single thread.

## Architecture

```
                        Your Runtime
┌──────────────────────────────────────────────────────┐
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │                 Executor                       │  │
│  │                                                │  │
│  │  task queue: [task_1, task_2, task_3, ...]     │  │
│  │                                                │  │
│  │  loop {                                        │  │
│  │      drain queue → poll each task              │  │
│  │      if queue empty → ask reactor to wait      │  │
│  │  }                                             │  │
│  └──────────────────────┬─────────────────────────┘  │
│                         │                            │
│                    wake(task)                         │
│                         │                            │
│  ┌──────────────────────┴─────────────────────────┐  │
│  │                  Reactor                        │  │
│  │                                                │  │
│  │  mio::Poll  ←──  kqueue / epoll               │  │
│  │  wakers: HashMap<Token, Waker>                 │  │
│  │                                                │  │
│  │  poll() → event on Token(3)                    │  │
│  │        → look up waker for Token(3)            │  │
│  │        → waker.wake() → task back in queue     │  │
│  └────────┬──────────┬───────────────────────────┘  │
│           │          │                               │
│     ┌─────┴───┐ ┌────┴────┐                         │
│     │Listener │ │ Stream  │  ← async wrappers       │
│     │Token(0) │ │Token(1) │    around std sockets    │
│     └────┬────┘ └────┬────┘    set to non-blocking   │
│          │           │                               │
└──────────┼───────────┼───────────────────────────────┘
           │           │
     ┌─────┴─────┐ ┌───┴───┐
     │ Client A  │ │Client B│    multiple concurrent
     │ nc ..8080 │ │nc ..   │    connections on one
     └───────────┘ └────────┘    thread
```

### Data flow for one echo

```
1. Client sends "hello\n" over TCP
2. kqueue/epoll fires → reactor sees Token(3) is READABLE
3. Reactor looks up waker for Token(3) → waker.wake()
4. Executor re-polls the task that owns Token(3)
5. Task calls stream.read() → gets "hello" (non-blocking, data is ready)
6. Task calls stream.write("hello") → registers WRITABLE if needed
7. Data goes back to the client
```

## What you implement

| Component       | Description                                              |
|-----------------|----------------------------------------------------------|
| `block_on`      | Runs the top-level future to completion on the current thread |
| `spawn`         | Wraps a future in a Task, pushes it to the executor queue |
| `Reactor`       | Owns `mio::Poll`, maps Tokens to Wakers, dispatches events |
| `TcpListener`   | Async wrapper around `mio::net::TcpListener`, returns `TcpStream` on accept |
| `TcpStream`     | Async read/write around `mio::net::TcpStream` with reactor registration |
| Event loop      | The main loop: drain tasks, poll reactor, repeat          |

## The goal

When you're done, this code runs:

```rust
my_runtime::block_on(async {
    let listener = my_runtime::TcpListener::bind("127.0.0.1:8080").await;
    loop {
        let stream = listener.accept().await;
        my_runtime::spawn(handle_client(stream));
    }
});

async fn handle_client(mut stream: my_runtime::TcpStream) {
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf).await;
        if n == 0 { return; }          // client disconnected
        stream.write_all(&buf[..n]).await;
    }
}
```

Multiple clients connect at the same time. Each gets its own task. All tasks
run concurrently on a single thread.

## Step-by-step implementation guide

### Step 1: Start with block_on (from Lesson 5)

Copy your `block_on` from Lesson 5. It should:

1. Pin the future
2. Create a waker that calls `thread::unpark()`
3. Loop: poll → Ready? return. Pending? `thread::park()`

At this point you can run a single future to completion.

### Step 2: Add the Reactor (from Lesson 8)

Create a global `Reactor` (use `thread_local!` or a `static` with `OnceLock`):

```rust
struct Reactor {
    poll: mio::Poll,
    wakers: HashMap<Token, Waker>,
    next_token: usize,
}
```

It needs three methods:
- `register(source, interest) -> Token` -- register a socket, return a token
- `set_waker(token, waker)` -- store the waker for a token
- `wait()` -- call `poll.poll()`, then for each event, call the stored waker

### Step 3: Implement TcpListener

Wrap `mio::net::TcpListener`:

```rust
struct TcpListener {
    inner: mio::net::TcpListener,
    token: Token,
}
```

`bind()` creates the listener, registers it with the reactor for READABLE.
`accept()` returns a future that:
- Tries `inner.accept()` -- if it returns a connection, wrap it in TcpStream
  and return `Ready`
- If `WouldBlock` -- store the waker with the reactor, return `Pending`

### Step 4: Implement TcpStream

Wrap `mio::net::TcpStream`:

```rust
struct TcpStream {
    inner: mio::net::TcpStream,
    token: Token,
}
```

`read()` and `write_all()` are futures:
- Try the operation -- if it succeeds, return `Ready`
- If `WouldBlock` -- register waker, return `Pending`

### Step 5: Add spawn + task queue

Add a shared task queue (just like Lesson 5's multi-task executor):

```rust
static TASK_QUEUE: Mutex<VecDeque<Arc<Task>>> = ...;
```

`spawn(future)` wraps the future in a `Task`, pushes it to the queue.

Update `block_on` to also drain the task queue on each iteration:
1. Poll the main future
2. Drain and poll all queued tasks
3. If nothing is ready, call `reactor.wait()` (which blocks until an event)
4. Repeat

### Step 6: Wire reactor events to wakers

This is where it all connects. When `reactor.wait()` fires:
- It gets events from `mio::Poll`
- For each event, it finds the stored waker and calls `waker.wake()`
- `wake()` pushes the task back into the queue
- The executor loop picks it up and polls it

Test the full loop:
1. Start the server
2. Connect with `nc`
3. Type a message
4. See it echoed back
5. Connect a second client -- both work concurrently

## Testing

### Manual testing with netcat

Terminal 1 -- start the server:
```bash
cargo run -p async-lessons --bin p1-echo-server -- run
```

Terminal 2 -- connect a client:
```bash
nc 127.0.0.1 8080
hello           # type this
hello           # server echoes it back
```

Terminal 3 -- connect another client simultaneously:
```bash
echo "world" | nc 127.0.0.1 8080
world           # echoed back
```

### Automated self-test

The `test` subcommand spawns the server, connects a client, sends data, and
checks the echo:

```bash
cargo run -p async-lessons --bin p1-echo-server -- test
```

## Exercises

### Exercise 1: Basic echo server

Implement all the components and get the goal code running. One client at a
time is fine for this exercise. Confirm with `nc`.

**Success criteria**: type a line into `nc`, see it echoed back.

### Exercise 2: Multiple concurrent clients

Make `spawn` work so multiple clients are served concurrently. Open 3-4
`nc` sessions and verify they all echo independently without blocking each
other.

**Success criteria**: send messages from multiple `nc` sessions interleaved
-- each gets its own echo back immediately.

### Exercise 3: Add a timeout

Add an `AsyncTimer` future (backed by a background thread or the reactor's
poll timeout). Disconnect clients that send nothing for 10 seconds:

```rust
async fn handle_client(mut stream: my_runtime::TcpStream) {
    let mut buf = [0u8; 1024];
    loop {
        match my_runtime::timeout(Duration::from_secs(10), stream.read(&mut buf)).await {
            Ok(0) => return,
            Ok(n) => stream.write_all(&buf[..n]).await,
            Err(_timeout) => {
                eprintln!("client timed out");
                return;
            }
        }
    }
}
```

**Success criteria**: connect with `nc`, wait 10 seconds without typing,
connection drops.
