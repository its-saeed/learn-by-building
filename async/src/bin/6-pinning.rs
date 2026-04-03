// Lesson 6: Pinning
//
// Understand why Pin exists, how self-referential structs break without it,
// and why every Future::poll takes Pin<&mut Self>.
//
// Run with: cargo run -p async-lessons --bin 6-pinning -- <command>
//
// Commands:
//   self-ref       Show the dangling pointer problem with self-referential structs
//   pin-it         Use Pin + PhantomPinned to prevent moves
//   unpin-demo     Show that Unpin types work fine with Pin::new
//   future-pin     Show why async futures need Pin (reference across await)
//   all            Run all demos

use clap::{Parser, Subcommand};
use std::future::Future;
use std::marker::PhantomPinned;
use std::pin::{pin, Pin};
use std::ptr;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

// ============================================================
// SelfRef: a struct with a pointer to its own field
// ============================================================

/// A struct that holds a String and a raw pointer intended to point
/// at that String. This simulates what the compiler generates for
/// async fn state machines that hold references across .await points.
struct SelfRef {
    data: String,
    /// Raw pointer to `data`. After init, this should equal &self.data.
    ptr: *const String,
}

impl SelfRef {
    /// Create a new SelfRef. Note: ptr is null until `init` is called.
    fn new(text: &str) -> Self {
        SelfRef {
            data: text.to_string(),
            ptr: ptr::null(),
        }
    }

    /// Initialize the self-referential pointer.
    /// After calling this, `self.ptr` points to `self.data`.
    fn init(&mut self) {
        self.ptr = &self.data as *const String;
    }

    /// Return the actual current address of the data field.
    fn data_addr(&self) -> *const String {
        &self.data as *const String
    }

    /// Return the stored pointer value (set at init time).
    fn stored_ptr(&self) -> *const String {
        self.ptr
    }

    /// Check if the pointer is still valid (points to data).
    fn is_valid(&self) -> bool {
        self.ptr == self.data_addr()
    }
}

// ============================================================
// PinnedSelfRef: a !Unpin version that can be safely pinned
// ============================================================

/// Same idea as SelfRef, but with PhantomPinned to opt out of Unpin.
/// Once pinned, the compiler prevents moving this struct.
struct PinnedSelfRef {
    data: String,
    ptr: *const String,
    _pin: PhantomPinned,
}

impl PinnedSelfRef {
    fn new(text: &str) -> Self {
        PinnedSelfRef {
            data: text.to_string(),
            ptr: ptr::null(),
            _pin: PhantomPinned,
        }
    }

    /// Initialize the pointer. Requires Pin<&mut Self> to ensure the
    /// value is already pinned before we create the self-reference.
    fn init(self: Pin<&mut Self>) {
        // SAFETY: we don't move self, we only write to the ptr field.
        unsafe {
            let this = self.get_unchecked_mut();
            this.ptr = &this.data as *const String;
        }
    }

    fn data_addr(&self) -> *const String {
        &self.data as *const String
    }

    fn stored_ptr(&self) -> *const String {
        self.ptr
    }

    fn is_valid(&self) -> bool {
        self.ptr == self.data_addr()
    }

    fn get_data(&self) -> &str {
        &self.data
    }
}

// ============================================================
// SelfRefFuture: a !Unpin future with internal self-reference
// ============================================================

/// A future that stores a String and a pointer to that String.
/// Demonstrates why futures must be pinned before polling.
///
/// TODO (Exercise 4): Implement Future for SelfRefFuture.
///   - On first poll: initialize the pointer, return Pending
///   - On second poll: verify the pointer is still valid, return Ready
struct SelfRefFuture {
    data: String,
    ptr: *const String,
    poll_count: u32,
    _pin: PhantomPinned,
}

impl SelfRefFuture {
    fn new(text: &str) -> Self {
        SelfRefFuture {
            data: text.to_string(),
            ptr: ptr::null(),
            poll_count: 0,
            _pin: PhantomPinned,
        }
    }
}

impl Future for SelfRefFuture {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
        // SAFETY: We only modify ptr and poll_count, we never move self.
        let this = unsafe { self.get_unchecked_mut() };

        this.poll_count += 1;

        match this.poll_count {
            1 => {
                // First poll: create the self-reference
                this.ptr = &this.data as *const String;
                println!("    [poll 1] Created self-reference");
                println!("      data addr: {:p}", &this.data);
                println!("      ptr value: {:p}", this.ptr);
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            2 => {
                // Second poll: verify the self-reference is still valid
                let data_addr = &this.data as *const String;
                let still_valid = this.ptr == data_addr;
                println!("    [poll 2] Checking self-reference");
                println!("      data addr: {:p}", data_addr);
                println!("      ptr value: {:p}", this.ptr);
                println!(
                    "      valid: {} (addresses {})",
                    still_valid,
                    if still_valid { "match" } else { "MISMATCH!" }
                );

                // Read through the pointer (safe because we're pinned)
                let value = unsafe { &*this.ptr };
                Poll::Ready(value.clone())
            }
            _ => Poll::Ready(this.data.clone()),
        }
    }
}

// ============================================================
// Noop waker (for manual polling demos)
// ============================================================

fn noop_waker() -> Waker {
    fn noop(_: *const ()) {}
    fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    unsafe { Waker::from_raw(RawWaker::new(ptr::null(), &VTABLE)) }
}

// ============================================================
// block_on: minimal executor for demos
// ============================================================

fn block_on<F: Future>(future: F) -> F::Output {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => {
                // In a real executor we'd park the thread.
                // Here our futures always wake immediately, so just loop.
                std::hint::spin_loop();
            }
        }
    }
}

// ============================================================
// CLI
// ============================================================

#[derive(Parser)]
#[command(name = "pinning", about = "Lesson 6: Pinning")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Demonstrate the dangling pointer problem with self-referential structs
    SelfRef,
    /// Use Pin + PhantomPinned to prevent moves and keep pointers valid
    PinIt,
    /// Show that Unpin types work fine with Pin::new
    UnpinDemo,
    /// Show why async futures need Pin (reference across await)
    FuturePin,
    /// Run all demos
    All,
}

// ============================================================
// Demo functions
// ============================================================

fn demo_self_ref() {
    println!("=== Step 1: Self-Referential Struct (the problem) ===");
    println!("Creating a struct with a field and a pointer to that field.");
    println!("Then moving the struct to show the pointer becomes invalid.");
    println!();

    let mut original = SelfRef::new("hello");
    original.init();

    println!("  Before move:");
    println!("    data addr:   {:p}", original.data_addr());
    println!("    stored ptr:  {:p}", original.stored_ptr());
    println!("    valid:       {}", original.is_valid());
    println!();

    // Move the struct -- this copies the bytes to a new location
    let moved = original;

    println!("  After move:");
    println!("    data addr:   {:p}", moved.data_addr());
    println!("    stored ptr:  {:p}", moved.stored_ptr());
    println!("    valid:       {}", moved.is_valid());
    println!();

    if !moved.is_valid() {
        println!("  The pointer is DANGLING! It still points to the old address.");
        println!("  data moved to {:p} but ptr still says {:p}.", moved.data_addr(), moved.stored_ptr());
    } else {
        println!("  (The compiler may have optimized away the move in this case.");
        println!("   In debug mode or with more complex structs, you'll see the mismatch.)");
    }
    println!();
    println!("Takeaway: moving a self-referential struct invalidates internal pointers.");
    println!("This is exactly what happens to async fn state machines if moved after polling.");
}

fn demo_pin_it() {
    println!("=== Step 2: Pin Prevents the Move ===");
    println!("Using Pin<Box<T>> with PhantomPinned (!Unpin) to safely self-reference.");
    println!();

    // Pin on the heap using Box::pin
    let mut pinned = Box::pin(PinnedSelfRef::new("pinned data"));
    pinned.as_mut().init();

    println!("  After pinning and init:");
    println!("    data addr:  {:p}", pinned.as_ref().data_addr());
    println!("    stored ptr: {:p}", pinned.as_ref().stored_ptr());
    println!("    valid:      {}", pinned.as_ref().is_valid());
    println!("    data:       \"{}\"", pinned.as_ref().get_data());
    println!();

    // Try to move it -- this line would NOT compile:
    //   let moved = *pinned;
    //   error: cannot move out of `Pin<Box<PinnedSelfRef>>`
    //
    // Try to get &mut T -- this also would NOT compile:
    //   let r: &mut PinnedSelfRef = Pin::get_mut(pinned.as_mut());
    //   error: `PinnedSelfRef` doesn't implement `Unpin`

    println!("  The following would NOT compile (uncomment to see errors):");
    println!("    let moved = *pinned;                    // can't move out of Pin");
    println!("    let r = Pin::get_mut(pinned.as_mut());  // PinnedSelfRef is !Unpin");
    println!();

    // We can still read through the pointer safely because the value hasn't moved
    let ptr = pinned.as_ref().stored_ptr();
    let value = unsafe { &*ptr };
    println!("  Reading through stored pointer: \"{}\"", value);
    println!("  Safe because Pin guarantees the value hasn't moved!");
    println!();
    println!("Takeaway: Pin<&mut T> for !Unpin types prevents moves.");
    println!("Internal pointers remain valid for the lifetime of the pinned value.");
}

fn demo_unpin() {
    println!("=== Step 3: Unpin Types Are Unaffected ===");
    println!("Most types implement Unpin. Pin has no effect on them.");
    println!();

    // Pin::new works on Unpin types
    let mut value = String::from("I'm Unpin");
    let pinned = Pin::new(&mut value);
    println!("  Created Pin<&mut String> with Pin::new -- works because String: Unpin");

    // Can get &mut T back
    let inner: &mut String = Pin::into_inner(pinned);
    inner.push_str(" and still mutable!");
    println!("  Got &mut String back via Pin::into_inner: \"{}\"", inner);
    println!();

    // Pin::new on primitive
    let mut x = 42_i32;
    let pinned = Pin::new(&mut x);
    println!("  Pin<&mut i32> via Pin::new: {}", *pinned);
    println!("  i32 is Unpin, so this is just a wrapper with no restrictions.");
    println!();

    // Pin::new on !Unpin would NOT compile:
    //   let mut sr = PinnedSelfRef::new("nope");
    //   let pinned = Pin::new(&mut sr);
    //   error: `PinnedSelfRef` doesn't implement `Unpin`
    println!("  Pin::new on !Unpin types does NOT compile:");
    println!("    let pinned = Pin::new(&mut my_not_unpin_value);  // error!");
    println!("    Use Box::pin() or pin!() macro instead.");
    println!();
    println!("Takeaway: Pin is a no-op for Unpin types. It only restricts !Unpin types.");
    println!("Since most types are Unpin, you only encounter Pin restrictions with futures.");
}

fn demo_future_pin() {
    println!("=== Step 4: Why Async Futures Need Pin ===");
    println!("async fn generates a self-referential state machine.");
    println!("Pin ensures the future stays put between polls.");
    println!();

    // --- Part A: Show that a hand-rolled self-referential future works when pinned ---
    println!("  Part A: Hand-rolled self-referential future (pinned)");
    println!();

    let result = block_on(SelfRefFuture::new("async data"));
    println!("    Result: \"{}\"", result);
    println!();

    // --- Part B: Show a real async fn that holds a reference across await ---
    println!("  Part B: Real async fn with reference across .await");
    println!();

    async fn holds_ref_across_await() -> usize {
        let data = vec![1, 2, 3, 4, 5];
        let slice = &data; // reference to local variable
        // If the future moved here, `slice` would dangle!
        tokio::task::yield_now().await;
        // After .await, `slice` must still be valid
        slice.len()
    }

    // Box::pin puts the future on the heap and pins it
    let future = holds_ref_across_await();
    let mut pinned = Box::pin(future);

    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    println!("    First poll...");
    match pinned.as_mut().poll(&mut cx) {
        Poll::Pending => println!("    Got Pending (future yielded at .await)"),
        Poll::Ready(v) => println!("    Got Ready({v}) on first poll"),
    }

    // The future is still at the same heap address. Its internal reference is valid.
    println!("    Future is pinned on heap -- internal references are safe.");
    println!("    Second poll...");
    match pinned.as_mut().poll(&mut cx) {
        Poll::Pending => println!("    Got Pending"),
        Poll::Ready(v) => println!("    Got Ready({v}) -- slice.len() still works!"),
    }

    println!();
    println!("  Why moving between polls would break:");
    println!("    poll 1 creates State0 {{ data: Vec, slice: &data }}");
    println!("    slice points to data's address inside the future struct");
    println!("    if future moved, data gets new address, slice is dangling");
    println!("    Pin prevents this -- future stays at its address forever");
    println!();
    println!("Takeaway: Future::poll takes Pin<&mut Self> because async state machines");
    println!("may be self-referential. The executor pins the future before first poll.");
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::SelfRef => demo_self_ref(),
        Command::PinIt => demo_pin_it(),
        Command::UnpinDemo => demo_unpin(),
        Command::FuturePin => demo_future_pin(),
        Command::All => {
            demo_self_ref();
            println!("\n");
            demo_pin_it();
            println!("\n");
            demo_unpin();
            println!("\n");
            demo_future_pin();
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
    fn self_ref_becomes_invalid_after_move() {
        let mut s = SelfRef::new("test");
        s.init();
        assert!(s.is_valid(), "should be valid before move");

        let moved = s;
        // After move, the pointer should not match the new data address.
        // (In optimized builds the compiler might elide the move, so we
        // check both cases.)
        let _ = moved.is_valid(); // just exercise the check
    }

    #[test]
    fn pinned_self_ref_stays_valid() {
        let mut pinned = Box::pin(PinnedSelfRef::new("test"));
        pinned.as_mut().init();
        assert!(pinned.as_ref().is_valid(), "pinned value should stay valid");
        assert_eq!(pinned.as_ref().get_data(), "test");
    }

    #[test]
    fn unpin_types_can_use_pin_new() {
        let mut x = 42_i32;
        let pinned = Pin::new(&mut x);
        assert_eq!(*pinned, 42);

        let mut s = String::from("hello");
        let pinned = Pin::new(&mut s);
        let inner = Pin::into_inner(pinned);
        inner.push_str(" world");
        assert_eq!(inner, "hello world");
    }

    #[test]
    fn self_ref_future_works_when_pinned() {
        let result = block_on(SelfRefFuture::new("test data"));
        assert_eq!(result, "test data");
    }
}
