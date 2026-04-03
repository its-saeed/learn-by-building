// Lesson 2: State Machines
// Manually desugar an async function into its state machine form.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// This is what we're desugaring:
//
// async fn add_slowly(a: u32, b: u32) -> u32 {
//     let x = yield_once(a).await;
//     let y = yield_once(b).await;
//     x + y
// }

enum AddSlowly {
    // TODO: define the states
    // State0: initial, holds a and b
    // State1: first await done, holds b and x
    // State2: both awaits done, holds x and y
    Start { a: u32, b: u32 },
}

impl Future for AddSlowly {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        // TODO: implement state transitions
        todo!()
    }
}

fn main() {
    // TODO: run AddSlowly to completion, print the result
    todo!()
}
