# Project 4: Async Job Queue with Workers, Backpressure, Graceful Shutdown

## What you'll learn

- Designing a multi-stage async pipeline
- Backpressure with bounded channels
- `spawn_blocking` for CPU-bound work
- Cancellation safety across the pipeline
- Testing async systems with time mocking

## Specification

### Architecture

```
Producer(s)
    |
    v
bounded mpsc channel (backpressure)
    |
    v
Dispatcher
    |
    +---> Worker 1 (spawn_blocking for CPU work)
    +---> Worker 2
    +---> Worker N
    |
    v
Results channel --> Result collector
```

### Job definition

```rust
struct Job {
    id: u64,
    payload: Vec<u8>,
    job_type: JobType,
}

enum JobType {
    CpuBound { iterations: u64 },   // use spawn_blocking
    AsyncIo { url: String },         // use async I/O
    Delayed { delay: Duration },     // sleep then execute
}

struct JobResult {
    job_id: u64,
    duration: Duration,
    status: ResultStatus,
}
```

### Components

| Component | Responsibility |
|-----------|---------------|
| Producer | Generates jobs, sends to bounded channel; blocks on backpressure |
| Dispatcher | Receives jobs, assigns to workers via `Semaphore(N)` |
| Worker | Executes job; uses `spawn_blocking` for CPU-bound work |
| Collector | Receives results, tracks stats, reports progress |
| Shutdown coordinator | `CancellationToken` hierarchy; drain pattern |

### Cancellation safety

- Dispatcher uses cancellation-safe `recv()` in `select!`
- Workers check cancellation between stages
- On shutdown: stop accepting new jobs, drain in-flight, collect final results

### Testing requirements

```rust
#[tokio::test]
async fn test_backpressure() {
    // Slow workers, fast producer -> producer blocks
}

#[tokio::test]
async fn test_graceful_shutdown() {
    tokio::time::pause();
    // Submit jobs, cancel token, advance time, verify all complete
}

#[tokio::test]
async fn test_worker_pool_concurrency() {
    // Submit N jobs, verify at most W run concurrently
}
```

## Key concepts

- **Bounded channel** for Producer -> Dispatcher backpressure
- **Semaphore** in Dispatcher to limit concurrent workers
- **spawn_blocking** for `JobType::CpuBound` to avoid starving async workers
- **CancellationToken** hierarchy: root -> dispatcher -> workers
- **time::pause + advance** for deterministic testing of delays and timeouts

## Exercises

1. Implement the basic pipeline: Producer -> Channel -> Dispatcher -> Workers -> Collector
2. Add backpressure: bounded channel with capacity 10, verify producer blocks when queue is full
3. Implement `spawn_blocking` for CPU-bound jobs; verify async workers are not starved
4. Add graceful shutdown with `CancellationToken` and drain pattern
5. Write tests using `time::pause()` for the `Delayed` job type
6. Add a dead-letter queue for failed jobs with retry logic (max 3 retries, exponential backoff)
7. Add metrics: jobs/sec, average latency, queue depth, worker utilization
