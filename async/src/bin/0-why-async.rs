// Lesson 0: Why Async?
//
// This program benchmarks threads vs async tasks to show why async matters.
// Run with: cargo run -p async-lessons --bin 0-why-async -- <command>
//
// Commands:
//   threads <count>      Spawn N threads, each sleeping 1s. Measure time + memory.
//   async <count>        Spawn N async tasks, each sleeping 1s. Measure time + memory.
//   max-threads          Keep spawning threads until the OS refuses.
//   compare <count>      Run both threads and async back-to-back, print comparison.

use clap::{Parser, Subcommand};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "why-async", about = "Benchmark threads vs async tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Spawn N OS threads, each sleeping 1s
    Threads { count: usize },
    /// Spawn N async tasks, each sleeping 1s
    Async { count: usize },
    /// Keep spawning threads until the OS refuses
    MaxThreads,
    /// Run both threads and async, print comparison table
    Compare { count: usize },
}

/// Spawn `count` OS threads, each sleeping for 1 second.
/// Returns (wall_time, peak_thread_count).
///
/// TODO: Implement this function.
///   1. Create an Arc<AtomicUsize> to track how many threads are alive
///   2. Spawn `count` threads. Each thread should:
///      a. Increment the alive counter
///      b. Sleep for 1 second (std::thread::sleep)
///      c. Decrement the alive counter
///   3. Join all threads
///   4. Return the elapsed time and peak alive count
fn bench_threads(count: usize) -> (Duration, usize) {
    todo!("Implement bench_threads")
}

/// Spawn `count` async tasks on tokio, each sleeping for 1 second.
/// Returns (wall_time, peak_task_count).
///
/// TODO: Implement this function.
///   1. Build a tokio runtime with Runtime::new()
///   2. Inside runtime.block_on(), spawn `count` tasks
///   3. Each task should:
///      a. Increment an alive counter (Arc<AtomicUsize>)
///      b. tokio::time::sleep for 1 second
///      c. Decrement the alive counter
///   4. Await all JoinHandles
///   5. Return the elapsed time and peak alive count
fn bench_async(count: usize) -> (Duration, usize) {
    todo!("Implement bench_async")
}

/// Keep spawning threads until the OS refuses.
/// Returns the max number of threads created.
///
/// TODO: Implement this function.
///   1. In a loop, spawn threads that sleep for 10 seconds (to keep them alive)
///   2. Catch the error when spawn fails (it returns Result)
///   3. Print the error message
///   4. Return the count
///
///   Hint: std::thread::Builder::new().spawn() returns Result, unlike
///   std::thread::spawn() which panics on failure.
fn find_max_threads() -> usize {
    todo!("Implement find_max_threads")
}

/// Print a formatted comparison of threads vs async.
///
/// TODO: Implement this function.
///   1. Call bench_threads(count)
///   2. Call bench_async(count)
///   3. Print a comparison table:
///      - Wall time for each
///      - Peak concurrent count for each
///      - Memory: run `ps -o rss -p <pid>` during each bench to estimate
///        (or just note the theoretical difference: count * 8MB vs count * ~100B)
fn compare(count: usize) {
    todo!("Implement compare")
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Threads { count } => {
            println!("Spawning {} threads...", count);
            println!("PID: {} — check memory with: ps -o pid,rss,vsz -p {}", std::process::id(), std::process::id());
            println!();
            let (elapsed, peak) = bench_threads(count);
            println!("Wall time: {:?}", elapsed);
            println!("Peak alive: {}", peak);
        }
        Command::Async { count } => {
            println!("Spawning {} async tasks...", count);
            println!("PID: {} — check memory with: ps -o pid,rss,vsz -p {}", std::process::id(), std::process::id());
            println!();
            let (elapsed, peak) = bench_async(count);
            println!("Wall time: {:?}", elapsed);
            println!("Peak alive: {}", peak);
        }
        Command::MaxThreads => {
            println!("Spawning threads until the OS says no...");
            let max = find_max_threads();
            println!("Max threads: {}", max);
        }
        Command::Compare { count } => {
            compare(count);
        }
    }
}

// ============================================================
// Tests — run with: cargo test -p async-lessons --bin 0-why-async
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_threads_completes() {
        let (elapsed, peak) = bench_threads(10);
        assert!(elapsed >= Duration::from_millis(900), "Should take ~1 second");
        assert!(elapsed < Duration::from_secs(5), "Shouldn't take too long");
        assert_eq!(peak, 10, "All 10 threads should be alive at once");
    }

    #[test]
    fn test_bench_async_completes() {
        let (elapsed, peak) = bench_async(10);
        assert!(elapsed >= Duration::from_millis(900), "Should take ~1 second");
        assert!(elapsed < Duration::from_secs(5), "Shouldn't take too long");
        assert_eq!(peak, 10, "All 10 tasks should be alive at once");
    }

    #[test]
    fn test_async_is_faster_than_threads_at_scale() {
        let (thread_time, _) = bench_threads(1000);
        let (async_time, _) = bench_async(1000);
        assert!(thread_time >= Duration::from_millis(900));
        assert!(async_time >= Duration::from_millis(900));
    }
}
