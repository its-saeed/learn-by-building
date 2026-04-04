// Lesson 12: Timers
//
// Add time awareness to your async runtime: timer heap, sleep(), timeout().
// Run with: cargo run -p async-lessons --bin 12-timers -- <command>
//
// Commands:
//   heap          Demo the timer heap: push, peek, fire expired
//   sleep-order   Spawn three sleeps, verify they fire in order
//   timeout       Demo the timeout combinator
//   all           Run all demos

use clap::{Parser, Subcommand};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

// ============================================================
// TimerEntry + TimerHeap
// ============================================================

/// A timer entry: a deadline and the waker to call when it expires.
struct TimerEntry {
    deadline: Instant,
    waker: Waker,
}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool { self.deadline == other.deadline }
}
impl Eq for TimerEntry {}
impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}
impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.deadline.cmp(&other.deadline) }
}

/// A min-heap of timer entries. The soonest deadline is at the top.
///
/// TODO: Implement the three methods below.
struct TimerHeap {
    heap: BinaryHeap<Reverse<TimerEntry>>,
}

impl TimerHeap {
    fn new() -> Self {
        Self { heap: BinaryHeap::new() }
    }

    /// Add a timer that fires at `deadline`.
    ///
    /// TODO: push a Reverse(TimerEntry { deadline, waker }) onto the heap.
    fn push(&mut self, deadline: Instant, waker: Waker) {
        todo!("Implement TimerHeap::push")
    }

    /// How long until the next timer fires?
    /// Returns None if no timers are pending.
    ///
    /// TODO: peek the heap, compute deadline - now (saturating).
    fn next_timeout(&self) -> Option<Duration> {
        todo!("Implement TimerHeap::next_timeout")
    }

    /// Wake all timers whose deadline has passed.
    ///
    /// TODO: pop entries while deadline <= now, call waker.wake().
    fn fire_expired(&mut self) {
        todo!("Implement TimerHeap::fire_expired")
    }

    fn len(&self) -> usize {
        self.heap.len()
    }
}

// ============================================================
// Sleep future
// ============================================================

/// A future that completes after a duration.
///
/// TODO: Implement Future for Sleep.
///   - If now >= deadline → Ready(())
///   - Else if not registered → push to timer heap, set registered, Pending
///   - Else → Pending (already registered, waiting for wake)
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

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // TODO: implement the three-way check described above.
        // For the timer heap, you'll need a global/thread-local TimerHeap.
        // For this exercise, just check the deadline:
        if Instant::now() >= self.deadline {
            Poll::Ready(())
        } else {
            // In a real implementation: push to timer heap here
            // For now, wake immediately (busy-poll, not efficient)
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Convenience function
fn sleep(duration: Duration) -> Sleep {
    Sleep::new(duration)
}

// ============================================================
// Timeout combinator
// ============================================================

/// Error returned when a timeout expires.
#[derive(Debug, PartialEq)]
struct TimedOut;

/// Wraps a future with a deadline. Returns Err(TimedOut) if the future
/// doesn't complete within the given duration.
///
/// TODO: Implement Future for Timeout.
///   1. Poll the inner future → if Ready, return Ok(value)
///   2. Poll the sleep → if Ready, return Err(TimedOut)
///   3. Both Pending → Pending
struct Timeout<F> {
    future: Pin<Box<F>>,
    sleep: Sleep,
}

impl<F: Future> Timeout<F> {
    fn new(duration: Duration, future: F) -> Self {
        Self {
            future: Box::pin(future),
            sleep: Sleep::new(duration),
        }
    }
}

impl<F: Future> Future for Timeout<F> {
    type Output = Result<F::Output, TimedOut>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: implement
        // Poll inner future first
        // Then poll sleep
        // If neither ready, return Pending
        todo!("Implement Timeout::poll")
    }
}

// ============================================================
// Simple executor (reused from earlier lessons)
// ============================================================

fn poll_to_completion<F: Future>(label: &str, mut future: F) -> F::Output {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(&waker);
    let mut pinned = unsafe { Pin::new_unchecked(&mut future) };
    let start = Instant::now();
    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Pending => {
                // Busy-poll with small sleep to avoid 100% CPU
                std::thread::sleep(Duration::from_millis(1));
            }
            Poll::Ready(output) => {
                let elapsed = start.elapsed();
                println!("  [{label}] Ready after {elapsed:.1?}");
                return output;
            }
        }
    }
}

// ============================================================
// CLI
// ============================================================

#[derive(Parser)]
#[command(name = "timers", about = "Lesson 12: Timers")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Demo the TimerHeap data structure
    Heap,
    /// Spawn three sleeps, verify ordering
    SleepOrder,
    /// Demo the timeout combinator
    Timeout,
    /// Run all demos
    All,
}

fn demo_heap() {
    println!("=== TimerHeap ===");
    println!("A min-heap of (deadline, waker) entries.");
    println!();

    let mut heap = TimerHeap::new();
    let waker = Waker::noop();

    let now = Instant::now();
    heap.push(now + Duration::from_secs(3), waker.clone());
    heap.push(now + Duration::from_secs(1), waker.clone());
    heap.push(now + Duration::from_secs(2), waker.clone());

    println!("  Pushed 3 timers: 3s, 1s, 2s");
    println!("  Heap size: {}", heap.len());

    let timeout = heap.next_timeout();
    println!("  Next timeout: {:?} (should be ~1s)", timeout);

    println!("  Waiting 1.1 seconds...");
    std::thread::sleep(Duration::from_millis(1100));

    heap.fire_expired();
    println!("  After fire_expired: {} timers remaining (should be 2)", heap.len());

    println!();
    println!("Takeaway: the heap always gives you the nearest deadline.");
    println!("fire_expired() wakes all timers that have passed.");
}

fn demo_sleep_order() {
    println!("=== Sleep Ordering ===");
    println!("Three sleeps: 300ms, 100ms, 200ms. Should complete in order.");
    println!();

    let start = Instant::now();

    // Run sequentially for simplicity (a real executor would interleave)
    poll_to_completion("100ms", sleep(Duration::from_millis(100)));
    poll_to_completion("200ms", sleep(Duration::from_millis(200)));
    poll_to_completion("300ms", sleep(Duration::from_millis(300)));

    let total = start.elapsed();
    println!();
    println!("  Total: {total:.1?} (sequential: ~600ms)");
    println!("  With a real executor + timer heap, these would be concurrent (~300ms).");
    println!();
    println!("Takeaway: sleep() + timer heap + reactor timeout = efficient async delays.");
}

fn demo_timeout() {
    println!("=== Timeout Combinator ===");
    println!("Wrapping a slow future in a fast timeout.");
    println!();

    println!("  timeout(200ms, sleep(100ms)) — should succeed:");
    let result = poll_to_completion("ok", Timeout::new(
        Duration::from_millis(200),
        sleep(Duration::from_millis(100)),
    ));
    println!("  Result: {:?}", result);

    println!();
    println!("  timeout(100ms, sleep(500ms)) — should time out:");
    let result = poll_to_completion("timeout", Timeout::new(
        Duration::from_millis(100),
        sleep(Duration::from_millis(500)),
    ));
    println!("  Result: {:?}", result);

    println!();
    println!("Takeaway: timeout races two futures — the inner one and a sleep.");
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Heap => demo_heap(),
        Command::SleepOrder => demo_sleep_order(),
        Command::Timeout => demo_timeout(),
        Command::All => {
            demo_heap();
            println!("\n");
            demo_sleep_order();
            println!("\n");
            demo_timeout();
        }
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleep_completes_after_duration() {
        let start = Instant::now();
        poll_to_completion("test", sleep(Duration::from_millis(50)));
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(45), "Should wait ~50ms");
        assert!(elapsed < Duration::from_millis(200), "Shouldn't take too long");
    }

    #[test]
    fn timer_heap_ordering() {
        let mut heap = TimerHeap::new();
        let waker = Waker::noop();
        let now = Instant::now();

        heap.push(now + Duration::from_secs(5), waker.clone());
        heap.push(now + Duration::from_secs(1), waker.clone());
        heap.push(now + Duration::from_secs(3), waker.clone());

        let timeout = heap.next_timeout().unwrap();
        assert!(timeout <= Duration::from_secs(1), "Nearest should be ~1s, got {:?}", timeout);
    }
}
