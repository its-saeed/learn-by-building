# Lesson 3: Wakers & Waking

## The problem

When a future returns `Pending`, the executor needs to know when to poll it again. It can't just poll everything in a busy loop — that wastes CPU.

## The solution: Wakers

A `Waker` is a callback handle. When a future returns `Pending`, it stores the waker. When the underlying I/O or timer is ready, it calls `waker.wake()`. This tells the executor: "hey, poll this task again."

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
    if self.is_ready() {
        Poll::Ready(())
    } else {
        // Save the waker so we can call it later
        self.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

// Later, when the event fires:
if let Some(waker) = self.waker.take() {
    waker.wake(); // tells executor to re-poll this task
}
```

## How Waker works internally

A `Waker` is built from a `RawWaker` — a fat pointer (data pointer + vtable):

```rust
struct RawWaker {
    data: *const (),           // pointer to the task/executor state
    vtable: &'static RawWakerVTable,  // function pointers: clone, wake, drop
}

struct RawWakerVTable {
    clone: unsafe fn(*const ()) -> RawWaker,
    wake: unsafe fn(*const ()),
    wake_by_ref: unsafe fn(*const ()),
    drop: unsafe fn(*const ()),
}
```

The executor provides the vtable implementation. When `waker.wake()` is called, it invokes the vtable's `wake` function, which typically puts the task back in the executor's run queue.

## Noop waker

For testing, you can create a waker that does nothing:

```rust
use std::task::{RawWaker, RawWakerVTable, Waker};

fn noop_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker { noop_raw_waker() }
    RawWaker::new(std::ptr::null(), &RawWakerVTable::new(clone, no_op, no_op, no_op))
}

fn noop_waker() -> Waker {
    unsafe { Waker::from_raw(noop_raw_waker()) }
}
```

## Exercises

### Exercise 1: Noop waker
Build a noop waker from `RawWaker` + vtable. Use it to manually poll the `CountdownFuture` from Lesson 1.

### Exercise 2: Counting waker
Build a waker that increments an `Arc<AtomicU32>` every time `wake()` is called. Poll a future and verify the waker was called the expected number of times.

### Exercise 3: Thread waker
Build a waker that unparks a specific thread (`std::thread::Thread::unpark()`). Park the current thread, have another thread call `waker.wake()`, verify the main thread resumes. This is how real single-threaded executors work.
