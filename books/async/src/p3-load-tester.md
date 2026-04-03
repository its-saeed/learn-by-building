# Project 3: HTTP Load Tester (mini wrk/hey)

## What you'll learn

- Building a real concurrent CLI tool with Tokio
- Controlling concurrency with `Semaphore`
- Graceful Ctrl+C handling with `CancellationToken`
- Collecting and reporting latency statistics (p50, p90, p99)

## Specification

### CLI interface

```
load-tester --url https://example.com/api --requests 1000 --concurrency 50
```

| Flag | Default | Description |
|------|---------|-------------|
| `--url` | required | Target URL |
| `--requests` | 100 | Total number of requests |
| `--concurrency` | 10 | Max concurrent requests |

### Architecture

```
main()
 +-- parse CLI args (clap)
 +-- create Semaphore(concurrency)
 +-- create CancellationToken
 +-- spawn signal handler (Ctrl+C -> cancel token)
 +-- spawn N request tasks, each:
 |    +-- acquire semaphore permit
 |    +-- select! { cancelled, send_request }
 |    +-- record latency + status
 +-- collect results
 +-- print report
```

### Output report

```
Requests:     1000 total, 985 succeeded, 15 failed
Duration:     2.34s
Throughput:   427.35 req/s

Latency:
  p50:    4.2ms
  p90:   12.1ms
  p99:   45.3ms
  max:   102.7ms

Status codes:
  200: 985
  503: 15
```

## Key concepts

- **Semaphore for concurrency** — each request acquires a permit before sending
- **CancellationToken** — Ctrl+C triggers cancellation; in-flight requests finish, no new ones start
- **Latency collection** — store `Duration` per request, sort, pick percentiles
- **HTTP client** — use `reqwest` or raw `hyper`; reuse the client for connection pooling

## Exercises

1. Implement the basic load tester with the CLI interface above
2. Add `--duration` mode: send requests for N seconds instead of a fixed count
3. Add `--method` and `--body` flags for POST/PUT testing
4. Print a live progress bar showing completed/total requests
5. Add a `--rate` flag for fixed request-per-second rate limiting (token bucket)
6. Export results as JSON with `--output json`
