# Learn by Building

Deep-dive courses into systems programming concepts. Each course teaches a topic from first principles through hands-on Rust implementations — no black boxes, no hand-waving.

## Courses

| Course | Lessons | Projects | What you'll build |
|--------|---------|----------|-------------------|
| [TLS](tls/) | 13 lessons | Encrypted + authenticated echo server | TLS from cryptographic primitives: hashing, encryption, signatures, key exchange, certificates |
| [Async](async/) | 28 lessons, 4 projects | Runtime from scratch, chat server, load tester, job queue | Async internals: futures, wakers, executors, reactors, tokio deep dive, production patterns |

## Structure

Each course follows the same pattern:
1. **Theory** — a `.md` file explaining the concept with real-world scenarios
2. **Code** — a `.rs` skeleton with TODOs to implement
3. **Exercises** — progressively harder tasks in each lesson
4. **Projects** — combine multiple lessons into something real

## Getting started

```sh
# TLS course
cd tls
cargo run -p tls --bin 1-hash -- --file-path Cargo.toml

# Async course
cd async
cargo run -p async-lessons --bin 1-futures
```

## Prerequisites

- Rust fundamentals (ownership, traits, generics, lifetimes)
- Basic networking (TCP/UDP)
