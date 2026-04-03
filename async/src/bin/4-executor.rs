// Lesson 4: A Minimal Executor
// Build a single-threaded executor that can run multiple futures.

fn main() {
    // TODO:
    // 1. Implement block_on<F: Future>(future: F) -> F::Output
    //    - Create a waker that unparks the current thread
    //    - Loop: poll, if Pending → park, if Ready → return
    //
    // 2. Implement a multi-task Executor with:
    //    - spawn(future) — adds to task queue
    //    - run() — polls tasks round-robin until all complete
    //
    // 3. Test: spawn multiple CountdownFutures, run them concurrently
    todo!()
}
