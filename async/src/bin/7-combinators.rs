// Lesson 7: Combinators — join and select
//
// Implement join and select by hand to understand how async combinators work.
// Run with: cargo run -p async-lessons --bin 7-combinators -- <command>
//
// Commands:
//   join       Demo MyJoin polling two futures concurrently
//   select     Demo MySelect racing two futures, show the loser being dropped
//   join-all   Demo join_all with a Vec of futures
//   all        Run all demos

use clap::{Parser, Subcommand};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

// ============================================================
// CountdownFuture (reused from Lesson 1)
// ============================================================

/// A future that counts down from `count` to 0.
/// Each poll decrements the counter and returns Pending.
/// When count reaches 0, returns Ready with the label.
struct CountdownFuture {
    label: &'static str,
    count: u32,
}

impl Future for CountdownFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
        if self.count == 0 {
            Poll::Ready(format!("{} done!", self.label))
        } else {
            println!("  [{}] count={}, not ready yet", self.label, self.count);
            self.count -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// ============================================================
// NamedFuture — prints on Drop to demonstrate cancellation
// ============================================================

/// A future with a name that announces when it is dropped.
/// Used to demonstrate that select cancels (drops) the losing future.
struct NamedFuture {
    name: &'static str,
    count: u32,
}

impl Future for NamedFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
        if self.count == 0 {
            Poll::Ready(format!("{} finished!", self.name))
        } else {
            println!("  [{}] polls remaining: {}", self.name, self.count);
            self.count -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl Drop for NamedFuture {
    fn drop(&mut self) {
        println!("  [{}] DROPPED (cancelled!)", self.name);
    }
}

// ============================================================
// Either — return type for select
// ============================================================

#[derive(Debug, PartialEq)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

// ============================================================
// Exercise 1: MyJoin
// ============================================================

/// A future that polls two sub-futures and returns both results as a tuple.
///
/// How it works:
/// - Each poll, check if a_result is None → poll a. If Ready, store result.
/// - Each poll, check if b_result is None → poll b. If Ready, store result.
/// - When both results are Some, return Ready((a, b)).
/// - Otherwise return Pending.
struct MyJoin<A: Future, B: Future> {
    a: A,
    b: B,
    a_result: Option<A::Output>,
    b_result: Option<B::Output>,
}

impl<A: Future, B: Future> MyJoin<A, B> {
    fn new(a: A, b: B) -> Self {
        MyJoin {
            a,
            b,
            a_result: None,
            b_result: None,
        }
    }
}

impl<A: Future + Unpin, B: Future + Unpin> Future for MyJoin<A, B> {
    type Output = (A::Output, B::Output);

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Implement join logic
        //
        // 1. Get a mutable reference to self (safe because A and B are Unpin)
        //    let this = self.get_mut();
        // 2. If a_result is None, poll a. If Ready, store in a_result.
        //    if this.a_result.is_none() {
        //        if let Poll::Ready(val) = Pin::new(&mut this.a).poll(_cx) {
        //            this.a_result = Some(val);
        //        }
        //    }
        // 3. Same for b
        // 4. If both results are Some, take them out and return Ready((a, b))
        // 5. Otherwise, call _cx.waker().wake_by_ref() and return Pending
        todo!("Exercise 1: implement Future for MyJoin")
    }
}

// ============================================================
// Exercise 2: MySelect
// ============================================================

/// A future that polls two sub-futures and returns whichever finishes first.
/// The losing future is dropped (cancelled).
///
/// How it works:
/// - Each poll, poll a. If Ready, return Either::Left(result).
///   (b is dropped when MySelect is dropped)
/// - If a is Pending, poll b. If Ready, return Either::Right(result).
///   (a is dropped when MySelect is dropped)
/// - If both Pending, return Pending.
struct MySelect<A: Future, B: Future> {
    a: A,
    b: B,
}

impl<A: Future, B: Future> MySelect<A, B> {
    fn new(a: A, b: B) -> Self {
        MySelect { a, b }
    }
}

impl<A: Future + Unpin, B: Future + Unpin> Future for MySelect<A, B> {
    type Output = Either<A::Output, B::Output>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Implement select logic
        //
        // 1. Get a mutable reference to self
        //    let this = self.get_mut();
        // 2. Poll a. If Ready, return Ready(Either::Left(value))
        //    if let Poll::Ready(val) = Pin::new(&mut this.a).poll(_cx) {
        //        return Poll::Ready(Either::Left(val));
        //    }
        // 3. Poll b. If Ready, return Ready(Either::Right(value))
        // 4. Both Pending → call _cx.waker().wake_by_ref() and return Pending
        //
        // When this future is dropped after returning Ready, the losing
        // future (still stored in the struct) is dropped too — that's cancellation!
        todo!("Exercise 2: implement Future for MySelect")
    }
}

// ============================================================
// Exercise 3: JoinAll
// ============================================================

/// A future that polls a Vec of futures and returns Vec of results.
/// Generalizes MyJoin from 2 futures to N futures.
///
/// How it works:
/// - Store futures as Vec<Option<F>> (None when done)
/// - Store results as Vec<Option<F::Output>>
/// - Each poll, iterate and poll any Some future
/// - When a future completes, store result and set future to None
/// - Return Ready when all results are Some
struct JoinAll<F: Future> {
    futures: Vec<Option<F>>,
    results: Vec<Option<F::Output>>,
}

impl<F: Future> JoinAll<F> {
    fn new(futures: Vec<F>) -> Self {
        let len = futures.len();
        JoinAll {
            futures: futures.into_iter().map(Some).collect(),
            results: (0..len).map(|_| None).collect(),
        }
    }
}

impl<F: Future + Unpin> Future for JoinAll<F> {
    type Output = Vec<F::Output>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Implement join_all logic
        //
        // 1. Get a mutable reference to self
        //    let this = self.get_mut();
        // 2. For each index, if futures[i] is Some, poll it
        //    - If Ready, store result in results[i] and set futures[i] = None
        //    for i in 0..this.futures.len() {
        //        if let Some(fut) = &mut this.futures[i] {
        //            if let Poll::Ready(val) = Pin::new(fut).poll(_cx) {
        //                this.results[i] = Some(val);
        //                this.futures[i] = None;
        //            }
        //        }
        //    }
        // 3. If all results are Some, collect them into a Vec and return Ready
        // 4. Otherwise, call _cx.waker().wake_by_ref() and return Pending
        todo!("Exercise 3: implement Future for JoinAll")
    }
}

// ============================================================
// Exercise 4: Map combinator
// ============================================================

/// A future that applies a function to the output of another future.
///
/// Map::new(some_future, |val| val * 2) → polls some_future, then doubles it.
struct Map<F: Future, Func> {
    future: F,
    func: Option<Func>,
}

impl<F: Future, Func> Map<F, Func> {
    fn new(future: F, func: Func) -> Self {
        Map {
            future,
            func: Some(func),
        }
    }
}

impl<F, Func, Out> Future for Map<F, Func>
where
    F: Future + Unpin,
    Func: FnOnce(F::Output) -> Out + Unpin,
{
    type Output = Out;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Implement map logic
        //
        // 1. Get a mutable reference to self
        //    let this = self.get_mut();
        // 2. Poll the inner future
        //    match Pin::new(&mut this.future).poll(_cx) {
        // 3. If Ready(value):
        //    - Take the function out of the Option (this.func.take().unwrap())
        //    - Apply it: (func)(value)
        //    - Return Ready(mapped_value)
        // 4. If Pending, return Pending
        //    }
        todo!("Exercise 4: implement Future for Map")
    }
}

// ============================================================
// Poll helper — manually drives a future to completion
// ============================================================

fn poll_to_completion<F: Future + Unpin>(label: &str, mut future: F) -> F::Output {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(&waker);

    let mut poll_count = 0;
    loop {
        poll_count += 1;
        match Pin::new(&mut future).poll(&mut cx) {
            Poll::Pending => {
                // continue polling
            }
            Poll::Ready(output) => {
                println!("  [{label}] Ready after {poll_count} polls");
                return output;
            }
        }
    }
}

// ============================================================
// CLI
// ============================================================

#[derive(Parser)]
#[command(name = "combinators", about = "Lesson 7: Combinators — join and select")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Demo MyJoin polling two futures concurrently
    Join,
    /// Demo MySelect racing two futures, show the loser being dropped
    Select,
    /// Demo join_all with a Vec of futures
    JoinAll,
    /// Run all demos
    All,
}

// ============================================================
// Demo functions
// ============================================================

fn demo_join() {
    println!("=== MyJoin: poll two futures, wait for BOTH ===");
    println!("Joining a 3-step countdown with a 5-step countdown...");
    println!();

    let a = CountdownFuture { label: "soup", count: 3 };
    let b = CountdownFuture { label: "salad", count: 5 };
    let joined = MyJoin::new(a, b);

    let (result_a, result_b) = poll_to_completion("join", joined);
    println!();
    println!("Results: ({result_a}, {result_b})");
    println!();
    println!("Takeaway: MyJoin polled both futures on every poll call.");
    println!("When soup finished first, it stopped polling soup but kept going on salad.");
    println!("Only when BOTH were done did it return Ready.");
}

fn demo_select() {
    println!("=== MySelect: poll two futures, take the FIRST, drop the loser ===");
    println!("Racing a 2-step future against a 5-step future...");
    println!();

    // Use NamedFuture so we see the Drop message
    let fast = NamedFuture { name: "taxi", count: 2 };
    let slow = NamedFuture { name: "bus", count: 5 };
    let selected = MySelect::new(fast, slow);

    let result = poll_to_completion("select", selected);
    // After poll_to_completion returns, `selected` is dropped.
    // The losing future's Drop impl will print a message.

    println!();
    match result {
        Either::Left(val) => println!("Winner: Left — {val}"),
        Either::Right(val) => println!("Winner: Right — {val}"),
    }
    println!();
    println!("Takeaway: the loser was DROPPED (cancelled) when select completed.");
    println!("You saw the drop message above. In real code, this means any");
    println!("partial work the loser did is lost. This is cancellation.");
}

fn demo_join_all() {
    println!("=== JoinAll: poll a Vec of futures, wait for ALL ===");
    println!("Joining 4 countdowns with different lengths...");
    println!();

    let futures = vec![
        CountdownFuture { label: "task-0", count: 2 },
        CountdownFuture { label: "task-1", count: 4 },
        CountdownFuture { label: "task-2", count: 1 },
        CountdownFuture { label: "task-3", count: 3 },
    ];
    let join_all = JoinAll::new(futures);

    let results = poll_to_completion("join_all", join_all);
    println!();
    println!("Results:");
    for (i, r) in results.iter().enumerate() {
        println!("  [{i}] {r}");
    }
    println!();
    println!("Takeaway: JoinAll generalizes MyJoin to any number of futures.");
    println!("Shorter futures finish first, but we wait for all of them.");
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Join => demo_join(),
        Command::Select => demo_select(),
        Command::JoinAll => demo_join_all(),
        Command::All => {
            demo_join();
            println!();
            demo_select();
            println!();
            demo_join_all();
        }
    }
}

// ============================================================
// Tests — run with: cargo test -p async-lessons --bin 7-combinators
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn poll_once<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
        let waker = Waker::noop();
        let mut cx = Context::from_waker(&waker);
        Pin::new(future).poll(&mut cx)
    }

    fn drive<F: Future + Unpin>(mut future: F) -> F::Output {
        let waker = Waker::noop();
        let mut cx = Context::from_waker(&waker);
        loop {
            match Pin::new(&mut future).poll(&mut cx) {
                Poll::Ready(val) => return val,
                Poll::Pending => {}
            }
        }
    }

    #[test]
    fn join_returns_both_results() {
        let a = CountdownFuture { label: "a", count: 2 };
        let b = CountdownFuture { label: "b", count: 3 };
        let joined = MyJoin::new(a, b);
        let (ra, rb) = drive(joined);
        assert_eq!(ra, "a done!");
        assert_eq!(rb, "b done!");
    }

    #[test]
    fn select_returns_faster_future() {
        let fast = CountdownFuture { label: "fast", count: 1 };
        let slow = CountdownFuture { label: "slow", count: 10 };
        let selected = MySelect::new(fast, slow);
        let result = drive(selected);
        // fast finishes in 2 polls (count=1 → Pending, count=0 → Ready)
        // slow needs 11 polls. On poll #2, fast is Ready, so Left wins.
        assert_eq!(result, Either::Left("fast done!".to_string()));
    }

    #[test]
    fn select_drops_loser() {
        use std::sync::atomic::{AtomicBool, Ordering};
        static WAS_DROPPED: AtomicBool = AtomicBool::new(false);

        struct DropDetector;
        impl Future for DropDetector {
            type Output = &'static str;
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                cx.waker().wake_by_ref();
                Poll::Pending // never finishes
            }
        }
        impl Drop for DropDetector {
            fn drop(&mut self) {
                WAS_DROPPED.store(true, Ordering::SeqCst);
            }
        }

        WAS_DROPPED.store(false, Ordering::SeqCst);

        let winner = CountdownFuture { label: "win", count: 1 };
        let loser = DropDetector;
        let selected = MySelect::new(winner, loser);
        let _result = drive(selected);

        assert!(WAS_DROPPED.load(Ordering::SeqCst), "loser future should have been dropped");
    }

    #[test]
    fn join_all_collects_all_results() {
        let futures = vec![
            CountdownFuture { label: "x", count: 1 },
            CountdownFuture { label: "y", count: 3 },
            CountdownFuture { label: "z", count: 2 },
        ];
        let results = drive(JoinAll::new(futures));
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], "x done!");
        assert_eq!(results[1], "y done!");
        assert_eq!(results[2], "z done!");
    }
}
