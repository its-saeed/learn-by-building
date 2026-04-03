# Async Rust & Tokio Internals

A hands-on course that builds async Rust from the ground up — from raw futures and wakers to a multi-threaded runtime, then into real tokio internals and production patterns.

## Prerequisites

- Rust fundamentals (ownership, traits, generics)
- TCP networking basics
- Completed [tls/](../tls/) or equivalent (helpful, not required)

## Courses

### Course 1: Async Fundamentals (no runtime, no tokio)

| # | Topic | Code | Notes |
|---|-------|------|-------|
| 0 | [Why Async?](src/bin/0-why-async.md) | [0-why-async.rs](src/bin/0-why-async.rs) | Threads vs async, C10K problem, benchmarking |
| 1 | [Futures by Hand](src/bin/1-futures.md) | [1-futures.rs](src/bin/1-futures.rs) | `Future` trait, `Poll`, `Pending`/`Ready` |
| 2 | [State Machines](src/bin/2-state-machines.md) | [2-state-machines.rs](src/bin/2-state-machines.rs) | What `async fn` compiles to |
| 3 | [Wakers](src/bin/3-wakers.md) | [3-wakers.rs](src/bin/3-wakers.rs) | `RawWaker`, vtable, waking mechanism |
| 4 | [A Minimal Executor](src/bin/4-executor.md) | [4-executor.rs](src/bin/4-executor.rs) | `block_on`, task queue, `spawn` |
| 5 | [Pinning](src/bin/5-pinning.md) | [5-pinning.rs](src/bin/5-pinning.rs) | `Pin`, self-referential structs, `Unpin` |
| 6 | [Combinators](src/bin/6-combinators.md) | [6-combinators.rs](src/bin/6-combinators.rs) | `join`, `select` — built by hand |
| 7 | [Async I/O Foundations](src/bin/7-async-io.md) | [7-async-io.rs](src/bin/7-async-io.rs) | kqueue/epoll, non-blocking sockets |

**Project 1**: [TCP Echo Server on your executor](src/bin/p1-echo-server.md) — [p1-echo-server.rs](src/bin/p1-echo-server.rs)

### Course 2: Build a Mini Tokio

| # | Topic | Code | Notes |
|---|-------|------|-------|
| 8 | [Event Loop + Reactor](src/bin/8-reactor.md) | [8-reactor.rs](src/bin/8-reactor.rs) | mio-based reactor, fd → waker mapping |
| 9 | [Task Scheduling](src/bin/9-task-scheduling.md) | [9-task-scheduling.rs](src/bin/9-task-scheduling.rs) | Task struct, JoinHandle, run queue |
| 10 | [AsyncRead / AsyncWrite](src/bin/10-async-read-write.md) | [10-async-read-write.rs](src/bin/10-async-read-write.rs) | Wrap non-blocking sockets in async traits |
| 11 | [Timers](src/bin/11-timers.md) | [11-timers.rs](src/bin/11-timers.rs) | Timer heap, `sleep()`, deadlines |
| 12 | [Channels](src/bin/12-channels.md) | [12-channels.rs](src/bin/12-channels.rs) | Async oneshot and mpsc |
| 13 | [Work-Stealing Scheduler](src/bin/13-work-stealing.md) | [13-work-stealing.rs](src/bin/13-work-stealing.rs) | Multi-threaded runtime |
| 14 | [Select Internals](src/bin/14-select.md) | [14-select.rs](src/bin/14-select.rs) | Race futures, cancellation, drop |

**Project 2**: [Multi-threaded Chat Server](src/bin/p2-chat-server.md) — [p2-chat-server.rs](src/bin/p2-chat-server.rs)

### Course 3: Tokio Deep Dive

| # | Topic | Code | Notes |
|---|-------|------|-------|
| 15 | [Tokio Architecture](src/bin/15-tokio-architecture.md) | [15-tokio-architecture.rs](src/bin/15-tokio-architecture.rs) | Runtime builder, drivers, scheduler |
| 16 | [Tokio I/O Driver](src/bin/16-tokio-io-driver.md) | [16-tokio-io-driver.rs](src/bin/16-tokio-io-driver.rs) | mio integration, Registration, readiness |
| 17 | [tokio::sync Internals](src/bin/17-tokio-sync.md) | [17-tokio-sync.rs](src/bin/17-tokio-sync.rs) | Mutex, Semaphore, Notify |
| 18 | [tokio::net](src/bin/18-tokio-net.md) | [18-tokio-net.rs](src/bin/18-tokio-net.rs) | TcpListener/TcpStream internals |
| 19 | [Task-Local Storage](src/bin/19-task-locals.md) | [19-task-locals.rs](src/bin/19-task-locals.rs) | Task-locals vs thread-locals |
| 20 | [Graceful Shutdown](src/bin/20-graceful-shutdown.md) | [20-graceful-shutdown.rs](src/bin/20-graceful-shutdown.rs) | CancellationToken, drain pattern |
| 21 | [Tracing & Debugging](src/bin/21-tracing.md) | [21-tracing.rs](src/bin/21-tracing.rs) | tokio-console, task dumps |

**Project 3**: [HTTP Load Tester](src/bin/p3-load-tester.md) — [p3-load-tester.rs](src/bin/p3-load-tester.rs)

### Course 4: Advanced Patterns

| # | Topic | Code | Notes |
|---|-------|------|-------|
| 22 | [Backpressure](src/bin/22-backpressure.md) | [22-backpressure.rs](src/bin/22-backpressure.rs) | Bounded channels, flow control |
| 23 | [Cancellation Safety](src/bin/23-cancellation.md) | [23-cancellation.rs](src/bin/23-cancellation.rs) | Dropped futures, data loss risks |
| 24 | [Sync ↔ Async Bridge](src/bin/24-sync-async-bridge.md) | [24-sync-async-bridge.rs](src/bin/24-sync-async-bridge.rs) | `block_on`, `spawn_blocking` |
| 25 | [Streams](src/bin/25-streams.md) | [25-streams.rs](src/bin/25-streams.rs) | Async iteration, `StreamExt` |
| 26 | [Connection Pooling](src/bin/26-connection-pool.md) | [26-connection-pool.rs](src/bin/26-connection-pool.rs) | Reuse, health checks, idle timeout |
| 27 | [Testing Async Code](src/bin/27-testing.md) | [27-testing.rs](src/bin/27-testing.rs) | Time mocking, deterministic testing |

**Project 4**: [Async Job Queue](src/bin/p4-job-queue.md) — [p4-job-queue.rs](src/bin/p4-job-queue.rs)

## How it all connects

```
Course 1: Async Fundamentals
  Futures → State Machines → Wakers → Executor → Pinning → I/O
                                         │
                                         ▼
                              Project 1: Echo Server
                              (your runtime from scratch)
                                         │
Course 2: Mini Tokio                     ▼
  Reactor → Tasks → AsyncRead → Timers → Channels → Work-Stealing → Select
                                         │
                                         ▼
                              Project 2: Chat Server
                              (multi-threaded, your runtime)
                                         │
Course 3: Tokio Deep Dive               ▼
  Architecture → I/O Driver → Sync → Net → Task-Locals → Shutdown → Tracing
                                         │
                                         ▼
                              Project 3: HTTP Load Tester
                              (real tokio internals)
                                         │
Course 4: Advanced Patterns              ▼
  Backpressure → Cancellation → Bridging → Streams → Pooling → Testing
                                         │
                                         ▼
                              Project 4: Job Queue
                              (production patterns)
```
