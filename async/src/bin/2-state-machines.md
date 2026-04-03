# Lesson 2: State Machines

## What `async fn` compiles to

When you write:

```rust
async fn fetch_data() -> String {
    let url = build_url().await;
    let response = http_get(url).await;
    response.body
}
```

The compiler transforms it into a state machine:

```rust
enum FetchData {
    Start,
    WaitingForUrl { /* locals captured before first await */ },
    WaitingForResponse { url: String /* locals captured before second await */ },
    Done,
}
```

Each `.await` point becomes a state transition. The struct holds all local variables that need to survive across await points.

## Why this matters

- **No heap allocation per await**: the state machine is one contiguous struct
- **No stack per task**: unlike threads, async tasks only store the data they need right now
- **Compiler-generated**: you get the efficiency of hand-written state machines with the readability of sequential code

## The size of futures

Every `async fn` generates a struct. Its size = size of the largest state variant. You can measure it:

```rust
println!("{}", std::mem::size_of_val(&some_future)); // often just a few dozen bytes
```

Compare with a thread stack: 8 MB vs ~100 bytes. This is why async scales.

## Exercises

### Exercise 1: Manual state machine
Write this async function:
```rust
async fn add_slowly(a: u32, b: u32) -> u32 {
    let x = yield_once(a).await;
    let y = yield_once(b).await;
    x + y
}
```

Then write the equivalent state machine struct by hand (no async keyword). Implement `Future` for it. Both should produce the same result.

### Exercise 2: cargo expand
Install `cargo-expand` (`cargo install cargo-expand`). Write a simple async function, run `cargo expand`, and read the generated state machine. Compare with your hand-written version.

### Exercise 3: Future size
Create several async functions of different complexity (0 awaits, 1 await, 5 awaits, one that holds a large `[u8; 1024]` across an await). Print the size of each future with `std::mem::size_of_val`. See how the size grows.
