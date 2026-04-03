# Lesson 4: A Minimal Executor

## What an executor does

An executor is the runtime that drives futures to completion. Its core loop:

```
1. Pick a task from the run queue
2. Poll it
3. If Ready → task is done, remove it
4. If Pending → the task's waker will re-queue it when ready
5. If no tasks are runnable → park (sleep) until a waker fires
6. Repeat
```

That's it. Everything else (I/O, timers, multi-threading) is built on top of this loop.

## The simplest executor

```rust
fn block_on<F: Future>(mut future: F) -> F::Output {
    let waker = /* create a waker that unparks the current thread */;
    let mut cx = Context::from_waker(&waker);
    loop {
        match Pin::new(&mut future).poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::park(), // sleep until woken
        }
    }
}
```

This runs ONE future. To run multiple futures concurrently, you need a task queue.

## Multi-task executor

```rust
struct Executor {
    queue: VecDeque<Box<dyn Future<Output = ()>>>,
}

impl Executor {
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        self.queue.push_back(Box::new(future));
    }

    fn run(&mut self) {
        while let Some(mut task) = self.queue.pop_front() {
            let waker = /* waker that pushes task back to queue */;
            match Pin::new(&mut task).poll(&mut cx) {
                Poll::Ready(()) => {} // done
                Poll::Pending => {} // waker will re-enqueue
            }
        }
    }
}
```

The key insight: the `Waker` closes over a reference to the task queue. When `wake()` is called, it pushes the task back into the queue so it gets polled again.

## Exercises

### Exercise 1: block_on
Implement `block_on` that runs a single future to completion. Use a thread-parking waker (from Lesson 3). Test with your `CountdownFuture`.

### Exercise 2: Multi-task executor
Implement `spawn()` and `run()`. Spawn 3 `CountdownFuture`s with different counts. Run them all to completion concurrently. Print when each finishes.

### Exercise 3: JoinHandle
Make `spawn()` return a `JoinHandle<T>` — a future that resolves to the spawned task's output. Implement it with a shared `Arc<Mutex<Option<T>>>` and a waker.
