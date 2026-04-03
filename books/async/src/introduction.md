# Async Rust & Tokio Internals

Build async Rust from the ground up — from raw futures and wakers to a multi-threaded runtime, then into real tokio internals and production patterns.

## Courses

| Course | Lessons | Project |
|--------|---------|---------|
| **1: Async Fundamentals** | 0-8 | TCP Echo Server on your executor |
| **2: Build a Mini Tokio** | 9-15 | Multi-threaded Chat Server |
| **3: Tokio Deep Dive** | 16-22 | HTTP Load Tester |
| **4: Advanced Patterns** | 23-28 | Async Job Queue |

## Prerequisites

- Rust fundamentals (ownership, traits, generics)
- TCP networking basics

## Source code

```sh
git clone https://github.com/its-saeed/learn-by-building.git
cd learn-by-building
cargo run -p async-lessons --bin 1-futures -- all
```
