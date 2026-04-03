// Lesson 3: Wakers & Waking
// Build a Waker from scratch using RawWaker + vtable.

use std::task::{RawWaker, RawWakerVTable, Waker, Context};

fn noop_raw_waker() -> RawWaker {
    // TODO: create a RawWaker with a no-op vtable
    // vtable needs: clone, wake, wake_by_ref, drop
    todo!()
}

fn noop_waker() -> Waker {
    // TODO: create a Waker from the RawWaker
    todo!()
}

fn main() {
    // TODO:
    // 1. Create a noop waker
    // 2. Create a Context from the waker
    // 3. Manually poll the CountdownFuture from Lesson 1
    // 4. Print each poll result
    todo!()
}
