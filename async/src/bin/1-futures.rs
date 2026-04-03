// Lesson 1: Futures by Hand
// Implement the Future trait manually — no async/await.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct CountdownFuture {
    count: u32,
}

impl Future for CountdownFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // TODO:
        // - If count > 0: decrement, wake the waker, return Pending
        // - If count == 0: return Ready(())
        todo!()
    }
}

fn main() {
    // TODO: poll CountdownFuture manually in a loop
    // (Lesson 3 shows how to create a waker, for now use a noop waker)
    todo!()
}
