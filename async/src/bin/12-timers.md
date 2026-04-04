# Lesson 12: Timers

> **Prerequisites**: Lesson 9 (Reactor), Lesson 10 (Task Scheduling). Timers integrate with the reactor's poll timeout and the executor's task queue.

## Real-life analogy: the kitchen timer rack

A chef has a rack of kitchen timers:

```
┌──────────────────────────────────────────────────────┐
│  Timer Rack (TimerHeap)                              │
│                                                      │
│  ⏰ Pasta: 8 min    (soonest → at the top)           │
│  ⏰ Sauce: 15 min                                    │
│  ⏰ Bread: 25 min                                    │
│                                                      │
│  Chef's loop:                                        │
│    1. Check: "which timer is closest?"  → Pasta (8m) │
│    2. Set a kitchen alarm for 8 minutes              │
│    3. Do other work until alarm rings                │
│    4. Alarm! → drain pasta                           │
│    5. Next closest: Sauce (15m) → set alarm for 7m  │
│    6. Continue...                                    │
└──────────────────────────────────────────────────────┘
```

The chef doesn't check every timer every second. They set ONE alarm for the nearest timer, then work on other things. When it rings, they handle it and set the next alarm.

This is exactly how async timers work:
- **Timer rack** = `BinaryHeap` of `(Instant, Waker)` entries
- **Nearest timer** = the heap's minimum (the `peek()`)
- **Kitchen alarm** = `mio::Poll::poll(timeout)` — the reactor blocks until this timeout
- **Alarm rings** = poll returns, we check for expired timers and wake them

## How timers integrate with the reactor

The reactor already has a `wait()` method that calls `poll.poll()`. We add a timeout:

```
                    Executor loop
                        │
                        ▼
        ┌───────────────────────────────┐
        │ Drain task queue, poll tasks  │
        │                               │
        │ Queue empty?                  │
        │   │                           │
        │   ▼                           │
        │ Check timer heap:             │
        │   Nearest deadline: 200ms     │
        │                               │
        │ reactor.wait(timeout: 200ms)  │──► mio::poll(200ms)
        │                               │    blocks at most 200ms
        │ ◄─────────────────────────────│
        │                               │
        │ Check expired timers:         │
        │   pasta timer expired → wake  │
        │                               │
        │ Back to draining queue        │
        └───────────────────────────────┘
```

Without timers, the reactor blocks forever (no timeout). With timers, the reactor blocks until either:
1. An I/O event fires, OR
2. The nearest timer expires

Whichever comes first.

## The TimerHeap

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::time::Instant;

struct TimerEntry {
    deadline: Instant,
    waker: Waker,
}

struct TimerHeap {
    heap: BinaryHeap<Reverse<TimerEntry>>,
}
```

We use `Reverse` so the heap is a **min-heap** — the soonest deadline is at the top.

Three operations:

```rust
impl TimerHeap {
    /// Add a timer. When deadline passes, the waker will be called.
    fn push(&mut self, deadline: Instant, waker: Waker) {
        self.heap.push(Reverse(TimerEntry { deadline, waker }));
    }

    /// How long until the next timer fires? Used as poll timeout.
    fn next_timeout(&self) -> Option<Duration> {
        self.heap.peek().map(|Reverse(entry)| {
            entry.deadline.saturating_duration_since(Instant::now())
        })
    }

    /// Wake all timers that have expired.
    fn fire_expired(&mut self) {
        let now = Instant::now();
        while let Some(Reverse(entry)) = self.heap.peek() {
            if entry.deadline <= now {
                let Reverse(entry) = self.heap.pop().unwrap();
                entry.waker.wake();
            } else {
                break;  // remaining timers are in the future
            }
        }
    }
}
```

## The Sleep future

```rust
struct Sleep {
    deadline: Instant,
    registered: bool,
}

impl Sleep {
    fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            registered: false,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if Instant::now() >= self.deadline {
            Poll::Ready(())
        } else {
            if !self.registered {
                timer_heap().push(self.deadline, cx.waker().clone());
                self.registered = true;
            }
            Poll::Pending
        }
    }
}
```

Usage: `sleep(Duration::from_secs(2)).await;`

### How it works end-to-end

```
1. Task calls sleep(2s).await
2. Sleep::poll() → deadline is in the future
   → push(deadline, waker) to timer heap
   → return Pending
3. Executor: queue empty
   → timer_heap.next_timeout() = 2s
   → reactor.wait(timeout: 2s)
   → mio::poll blocks for 2 seconds
4. poll returns (timeout expired)
   → timer_heap.fire_expired()
   → waker.wake() → task re-queued
5. Executor polls task again
   → Sleep::poll() → Instant::now() >= deadline
   → return Ready(())
```

## The timeout combinator

Wrap any future with a deadline:

```rust
async fn timeout<F: Future>(duration: Duration, future: F) -> Result<F::Output, TimedOut> {
    select! {
        result = future => Ok(result),
        _ = sleep(duration) => Err(TimedOut),
    }
}
```

Or without select, as a manual future:

```rust
struct Timeout<F> {
    future: F,
    sleep: Sleep,
}

impl<F: Future> Future for Timeout<F> {
    type Output = Result<F::Output, TimedOut>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // Poll the inner future first
        if let Poll::Ready(val) = self.future.poll(cx) {
            return Poll::Ready(Ok(val));
        }
        // Then check the timer
        if let Poll::Ready(()) = self.sleep.poll(cx) {
            return Poll::Ready(Err(TimedOut));
        }
        Poll::Pending
    }
}
```

## Exercises

### Exercise 1: Sleep and ordering

Implement `Sleep` and the `TimerHeap`. Spawn three tasks:
```rust
spawn(async { sleep(Duration::from_secs(3)).await; println!("C: 3s"); });
spawn(async { sleep(Duration::from_secs(1)).await; println!("A: 1s"); });
spawn(async { sleep(Duration::from_secs(2)).await; println!("B: 2s"); });
```

They should print in order: A, B, C. Total time should be ~3 seconds (concurrent), not 6 (sequential).

### Exercise 2: Timer integration with reactor

Modify your executor's main loop:
1. After draining the task queue, call `timer_heap.fire_expired()`
2. Compute `timer_heap.next_timeout()`
3. Pass it to `reactor.wait(timeout)`

Verify: spawn a sleep(1s) task alongside an I/O task. Both should work correctly — the reactor wakes up for either I/O events or timer expiry.

### Exercise 3: Timeout combinator

Implement `timeout(duration, future)`. Test:
```rust
let result = timeout(Duration::from_millis(100), sleep(Duration::from_secs(10))).await;
assert!(result.is_err());  // timed out!

let result = timeout(Duration::from_secs(10), sleep(Duration::from_millis(100))).await;
assert!(result.is_ok());   // completed in time
```

### Exercise 4: Interval

Implement `interval(duration)` that yields `()` at regular intervals. Use it to print a heartbeat every 500ms while another task does a 3-second sleep.

```rust
spawn(async {
    let mut i = interval(Duration::from_millis(500));
    loop {
        i.tick().await;
        println!("heartbeat");
    }
});
spawn(async {
    sleep(Duration::from_secs(3)).await;
    println!("done, shutting down");
    // TODO: cancel the heartbeat task
});
```
