# Lesson 1: Futures by Hand

## What is a Future?

A future is a value that might not be ready yet. It's Rust's core async abstraction:

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

That's it. A future has one method: `poll`. When polled:
- Return `Poll::Ready(value)` if the result is available
- Return `Poll::Pending` if not ready yet

The runtime calls `poll` repeatedly. When a future returns `Pending`, it promises to wake the runtime (via `cx.waker()`) when it might be ready to make progress.

## What `.await` does

```rust
let value = some_future.await;
```

This compiles to roughly:

```rust
loop {
    match some_future.poll(cx) {
        Poll::Ready(value) => break value,
        Poll::Pending => yield, // give control back to the runtime
    }
}
```

`.await` is syntactic sugar for "keep polling until ready, yielding between attempts."

## The contract

- A future must not be polled after returning `Ready` (undefined behavior)
- A future that returns `Pending` MUST arrange for `cx.waker().wake()` to be called, otherwise it will never be polled again (the task hangs forever)
- Polling should be cheap — do a small amount of work, then return

## Real-world analogy

Ordering food at a restaurant:
- **Blocking (threads)**: You stand at the counter and wait. Can't do anything else.
- **Polling (futures)**: You get a buzzer. You go do other things. The buzzer vibrates → your food is ready → you go pick it up.

The `Waker` is the buzzer.

## Exercises

### Exercise 1: CountdownFuture
Implement a future that counts down from N to 0. Each `poll` decrements the counter and returns `Pending`. When it hits 0, return `Ready(())`.

```rust
struct CountdownFuture {
    count: u32,
}

impl Future for CountdownFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // TODO
    }
}
```

Hint: you need to call `cx.waker().wake_by_ref()` when returning `Pending`, otherwise the executor won't poll you again.

### Exercise 2: ReadyFuture
Implement a future that immediately returns a value:
```rust
struct ReadyFuture<T>(Option<T>);
```
First poll returns `Ready(value)`. This is what `std::future::ready()` does.

### Exercise 3: Poll manually
Don't use an executor. Create a dummy waker (Lesson 3 will show how), create a `Context`, and call `poll()` on your `CountdownFuture` in a manual loop. See the state change with each poll.
