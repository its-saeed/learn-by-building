# Project 1: Mini Executor + TCP Echo Server

## Overview

Build a single-threaded async runtime from scratch that runs a real TCP echo server. No tokio, no external async runtime.

Combines: Lessons 1-7 (futures, wakers, executor, pinning, kqueue/epoll)

## What you implement

- `my_runtime::block_on(future)` — runs a future to completion
- `my_runtime::spawn(future)` — schedules a task
- `my_runtime::TcpListener` — async TCP listener backed by kqueue/mio
- `my_runtime::TcpStream` — async TCP stream with read/write
- The event loop / reactor that bridges kqueue events to wakers

## The goal

```rust
my_runtime::block_on(async {
    let listener = my_runtime::TcpListener::bind("127.0.0.1:8080").await;
    loop {
        let stream = listener.accept().await;
        my_runtime::spawn(handle_client(stream));
    }
});
```

Test with `nc 127.0.0.1 8080` — type a message, see it echoed back. Multiple concurrent clients should work.
