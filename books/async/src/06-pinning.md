# Lesson 6: Pinning

> **Prerequisites**: Lesson 5 (Executor) -- you should understand how an executor polls futures before learning why those futures must be pinned.

## Real-life analogy: the house with an address

Imagine you buy a house at **123 Elm Street**. Your friends, your bank, the post office -- everyone has your address written down. Now imagine a magical crane lifts your entire house and drops it on a different lot. Your friends show up at 123 Elm Street and find an empty plot. The mail goes nowhere. Every reference to your location is now wrong.

**Pin = a city ordinance that says "this house will not move."** Once pinned, everyone who has your address can trust it forever.

```
Before Pin:                          After Pin:
┌──────────────────────┐             ┌──────────────────────┐
│  House @ 123 Elm St  │             │  House @ 123 Elm St  │
│  (can be moved)      │             │  PINNED -- no moving │
│                      │  crane      │                      │
│  Friends: "123 Elm"  │──────X      │  Friends: "123 Elm"  │ -- always valid
│  Bank:    "123 Elm"  │             │  Bank:    "123 Elm"  │
└──────────────────────┘             └──────────────────────┘
```

In Rust terms:
- **House** = a value in memory
- **Address** = a pointer/reference to that value
- **Moving the house** = `mem::swap`, `mem::replace`, or just assignment to a different variable
- **Pin** = a wrapper that prevents you from getting `&mut T` (which would let you move the value)

## The problem: self-referential structs

A **self-referential struct** has a field that points to another field inside the same struct. This is the core problem Pin solves.

```rust
struct SelfRef {
    data: String,
    ptr: *const String,  // points to `data` above
}
```

After initialization, `ptr` points to `data`'s memory location:

```
                    SelfRef (at address 0x1000)
                    ┌─────────────────────────────────┐
                    │  data: String { "hello" }       │  <-- lives at 0x1000
                    │  ptr:  0x1000 ──────────────┐   │
                    │                             │   │
                    │                             ▼   │
                    │         (points to data) ───┘   │
                    └─────────────────────────────────┘
                    ptr == &data  -- correct!
```

Now move the struct (e.g., by returning it from a function, pushing to a Vec, or `mem::swap`):

```
    BEFORE MOVE                              AFTER MOVE

    0x1000 (old location)                    0x2000 (new location)
    ┌───────────────────────┐                ┌───────────────────────┐
    │  data: "hello"        │                │  data: "hello"        │  <-- now at 0x2000
    │  ptr:  0x1000 ────┐   │   ──move──►    │  ptr:  0x1000 ────┐   │
    │                   │   │                │                   │   │
    │                   ▼   │                │                   │   │
    │       (self)  ────┘   │                │       DANGLING!   │   │
    └───────────────────────┘                └───────────────────┘   │
                                                                    │
                                              0x1000 (old location) │
                                              ┌─────────────────┐   │
                                              │  (freed/garbage) │◄──┘
                                              └─────────────────┘
                                              ptr still points here!
```

The data moved to 0x2000, but `ptr` still says 0x1000. **Dangling pointer.** Undefined behavior.

### Concrete code example

```rust
use std::ptr;

struct SelfRef {
    data: String,
    ptr: *const String,
}

impl SelfRef {
    fn new(text: &str) -> Self {
        let mut s = SelfRef {
            data: text.to_string(),
            ptr: ptr::null(),
        };
        s.ptr = &s.data as *const String;
        s
    }

    fn data_addr(&self) -> *const String {
        &self.data as *const String
    }

    fn ptr_value(&self) -> *const String {
        self.ptr
    }
}

let mut a = SelfRef::new("hello");
a.ptr = &a.data;                       // fix pointer after construction
println!("a.data addr: {:p}", &a.data); // e.g., 0x1000
println!("a.ptr value: {:p}", a.ptr);   // 0x1000 -- matches!

let mut b = a;                          // MOVE a into b
println!("b.data addr: {:p}", &b.data); // e.g., 0x2000 (new location)
println!("b.ptr value: {:p}", b.ptr);   // still 0x1000 -- DANGLING!
```

## What Pin<&mut T> does

`Pin<&mut T>` wraps a mutable reference and **removes your ability to get `&mut T` back** (for non-Unpin types). Without `&mut T`, you cannot:

- `mem::swap` the value with another
- `mem::replace` the value
- Move it by assignment

```
    Normal &mut T:                   Pin<&mut T>:
    ┌──────────────────┐             ┌──────────────────┐
    │  &mut T           │             │  Pin<&mut T>      │
    │                  │             │                  │
    │  Can do:         │             │  Can do:         │
    │  - read          │             │  - read          │
    │  - write fields  │             │  - write fields  │
    │  - mem::swap !!  │             │                  │
    │  - move out !!   │             │  CANNOT:         │
    │                  │             │  - get &mut T    │
    └──────────────────┘             │  - mem::swap     │
                                     │  - move out      │
                                     └──────────────────┘
```

The signature of `Future::poll` requires `Pin<&mut Self>`:

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
```

This is not an accident. The executor must guarantee that once it starts polling a future, it will never move that future again.

## The Unpin trait

`Unpin` is an auto-trait that says "this type is safe to move even when pinned." It is a promise that the type has no self-references.

```
    Most types:              async fn futures:
    ┌─────────────────┐     ┌─────────────────────────┐
    │  i32             │     │  async fn foo() {       │
    │  String          │     │      let x = 1;         │
    │  Vec<T>          │     │      let r = &x;        │
    │  HashMap<K,V>    │     │      bar().await;       │
    │  ...             │     │      use(r);            │
    │                  │     │  }                      │
    │  All Unpin       │     │                         │
    │  Pin::new works  │     │  !Unpin (not Unpin)     │
    └─────────────────┘     │  needs Box::pin or pin! │
                             └─────────────────────────┘
```

- `Unpin` types: `Pin<&mut T>` freely gives you `&mut T` via `Pin::get_mut()`. Pin has no effect.
- `!Unpin` types: `Pin<&mut T>` does NOT give you `&mut T`. The value is truly pinned.

You can opt out of `Unpin` manually with `PhantomPinned`:

```rust
use std::marker::PhantomPinned;

struct MyStruct {
    data: String,
    _pin: PhantomPinned,  // makes MyStruct: !Unpin
}
```

## Pin in practice

### Pin::new -- only for Unpin types

```rust
let mut x = 42_i32;  // i32: Unpin
let pinned = Pin::new(&mut x);  // works fine
```

This does NOT work for `!Unpin` types -- the compiler will reject it.

### Box::pin -- heap-pin any value

```rust
let future = async { 42 };
let pinned: Pin<Box<dyn Future<Output = i32>>> = Box::pin(future);
// future is on the heap and cannot be moved
```

This is what executors use: `Pin<Box<dyn Future>>`.

### pin! macro -- stack-pin a value

```rust
use std::pin::pin;

let future = async { 42 };
let mut pinned = pin!(future);  // pins to the stack
// pinned: Pin<&mut impl Future<Output = i32>>
```

The macro shadows the binding so you cannot access the original (un-pinned) value.

## Why futures need Pin

An `async fn` compiles to a state machine struct. If the function holds a reference to a local variable across an `.await`, the struct becomes self-referential:

```rust
async fn process() {
    let buffer = vec![1, 2, 3];
    let slice = &buffer;          // reference to local
    network_send(slice).await;    // <-- .await here
    println!("{:?}", slice);      // slice must still be valid
}
```

The compiler generates something like:

```rust
enum ProcessFuture {
    // State 0: before the await
    State0 {
        buffer: Vec<u8>,
        slice: &Vec<u8>,         // <-- points to buffer above!
    },
    // State 1: after the await
    State1 {
        buffer: Vec<u8>,
        slice: &Vec<u8>,
    },
    Done,
}
```

`State0` is self-referential: `slice` points into `buffer`. If you move `ProcessFuture` after the first `poll()`, the `slice` pointer dangles.

```
    ProcessFuture after first poll (State0):

    ┌───────────────────────────────────────┐
    │  buffer: Vec [1, 2, 3]               │  <-- at address 0xA000
    │  slice:  &buffer ───────────────┐     │
    │                                 │     │
    │                                 ▼     │
    │              (points to 0xA000) ─┘     │
    └───────────────────────────────────────┘

    If moved to new address 0xB000:

    ┌───────────────────────────────────────┐
    │  buffer: Vec [1, 2, 3]               │  <-- now at 0xB000
    │  slice:  &buffer ───────────────┐     │
    │                                 │     │
    │                                 X     │  slice still says 0xA000
    │              DANGLING!          │     │  but buffer is at 0xB000
    └───────────────────────────────────────┘
```

This is why `Future::poll` takes `Pin<&mut Self>`: the executor pins the future before the first poll and never moves it again.

### The lifecycle

```
    1. Create future:       let f = process();           // future exists, not yet polled
    2. Pin it:              let f = Box::pin(f);         // pinned on heap, cannot move
    3. First poll:          f.as_mut().poll(cx);         // enters State0, creates self-ref
    4. Returns Pending:     slice points to buffer       // self-ref is valid
    5. ...                  (future stays at same address)
    6. Second poll:         f.as_mut().poll(cx);         // slice still valid!
    7. Returns Ready:       done
```

Without Pin, step 5 could be "move the future into a different Vec slot" and step 6 would be undefined behavior.

## Summary

| Concept | What it means |
|---------|--------------|
| Self-referential struct | A struct with a pointer to its own field |
| Move | Copies bytes to new location, old location invalid |
| Dangling pointer | Pointer to old location after a move |
| `Pin<&mut T>` | Wrapper that prevents getting `&mut T` (for !Unpin) |
| `Unpin` | Auto-trait: "safe to move when pinned" (most types) |
| `!Unpin` | Not safe to move when pinned (async fn futures) |
| `PhantomPinned` | Opt out of Unpin manually |
| `Pin::new` | Pin an Unpin type (no-op, just wraps) |
| `Box::pin` | Pin any type on the heap |
| `pin!` | Pin any type on the stack |

## Exercises

### Exercise 1: Demonstrate the dangling pointer

Create a `SelfRef` struct with a `String` and a `*const String` raw pointer. Initialize the pointer to point at the data field. Move the struct. Print the addresses before and after -- show that the pointer no longer matches the data's actual address.

### Exercise 2: Pin prevents the move

Use `Pin` and `PhantomPinned` to create a `!Unpin` struct. Try to move it after pinning. Observe the compiler error. Then show that you can safely read through the pointer because the value hasn't moved.

### Exercise 3: Unpin types are unaffected

Show that `Pin::new` works on Unpin types (like `String`, `i32`). Demonstrate that you can still get `&mut T` from `Pin<&mut T>` when `T: Unpin`, so Pin is effectively a no-op for these types.

### Exercise 4: Why async futures need Pin

Write an async function that holds a reference across an `.await`. Box::pin it, poll it manually with a waker, and show it works. Then explain (in a comment) why moving the future between polls would break the internal reference.
