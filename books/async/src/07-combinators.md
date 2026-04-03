# Lesson 7: Combinators — join and select

## Real-life analogy: cooking dinner

### Join = make salad AND soup, serve when BOTH are done

You're cooking dinner. You put soup on the stove and start chopping salad. You check the soup, chop some more, check the soup again. You keep switching between the two tasks. When **both** are done, you serve dinner.

```
You (the executor):

  "Is the soup done?"   → No, keep simmering     (Poll::Pending)
  "Is the salad done?"  → No, still chopping      (Poll::Pending)
  "Is the soup done?"   → No, keep simmering      (Poll::Pending)
  "Is the salad done?"  → Yes! Set it aside.       (Poll::Ready)
  "Is the soup done?"   → No, keep simmering      (Poll::Pending)
  "Is the soup done?"   → Yes! Take it off heat.   (Poll::Ready)

  Both done → serve dinner!  → return (soup, salad)
```

Join **waits for all futures to complete**. It returns a tuple of all results.

### Select = order a taxi AND a bus ticket, take whichever arrives first

You need to get across town. You order a taxi on your phone and walk to the bus stop. Whichever arrives first, you take it — and **cancel the other**.

```
You (the executor):

  "Has the taxi arrived?"  → No                    (Poll::Pending)
  "Has the bus arrived?"   → No                    (Poll::Pending)
  "Has the taxi arrived?"  → No                    (Poll::Pending)
  "Has the bus arrived?"   → Yes! Get on the bus.  (Poll::Ready)

  Bus won → cancel the taxi!  → DROP the taxi future
  return bus
```

Select **waits for the first future to complete**. It returns that result and **drops the loser**.

## How join works internally

`join!(a, b)` creates a single future that, each time it is polled, polls both `a` and `b`. It tracks which ones are done with flags. When all are `Ready`, it returns the collected results.

```
join!(soup, salad) — one combined future
─────────────────────────────────────────────────────────────────
  poll #1:
  ┌─────────────────────────────────────────────────────┐
  │  poll soup  → Pending    (soup_done = false)        │
  │  poll salad → Pending    (salad_done = false)       │
  │  → return Pending                                   │
  └─────────────────────────────────────────────────────┘

  poll #2:
  ┌─────────────────────────────────────────────────────┐
  │  poll soup  → Pending    (soup_done = false)        │
  │  poll salad → Ready(S)   (salad_done = true)        │
  │  → return Pending  (soup not done yet)              │
  └─────────────────────────────────────────────────────┘

  poll #3:
  ┌─────────────────────────────────────────────────────┐
  │  poll soup  → Ready(S)   (soup_done = true)         │
  │  skip salad (already done)                          │
  │  → return Ready((soup, salad))  ALL DONE!           │
  └─────────────────────────────────────────────────────┘
```

Key insight: `join` is **one future** that polls **multiple sub-futures**. There is no parallelism — it is concurrent on a single thread. Each call to `poll` on the join future polls each child that is not yet done.

### Pseudocode

```rust
struct MyJoin<A, B> {
    a: A,
    b: B,
    a_result: Option<A::Output>,
    b_result: Option<B::Output>,
}

fn poll(self, cx) -> Poll<(A::Output, B::Output)> {
    if self.a_result.is_none() {
        if let Ready(val) = self.a.poll(cx) {
            self.a_result = Some(val);
        }
    }
    if self.b_result.is_none() {
        if let Ready(val) = self.b.poll(cx) {
            self.b_result = Some(val);
        }
    }
    if self.a_result.is_some() && self.b_result.is_some() {
        Ready((self.a_result.take().unwrap(), self.b_result.take().unwrap()))
    } else {
        Pending
    }
}
```

## How select works internally

`select!(a, b)` creates a single future that polls both `a` and `b` each time. As soon as **one** returns `Ready`, it returns that value and **drops the other future**.

```
select!(taxi, bus) — one combined future
─────────────────────────────────────────────────────────────────
  poll #1:
  ┌─────────────────────────────────────────────────────┐
  │  poll taxi → Pending                                │
  │  poll bus  → Pending                                │
  │  → return Pending                                   │
  └─────────────────────────────────────────────────────┘

  poll #2:
  ┌─────────────────────────────────────────────────────┐
  │  poll taxi → Pending                                │
  │  poll bus  → Ready(bus_result)                      │
  │  → return Ready(Right(bus_result))                  │
  └─────────────────────────────────────────────────────┘

  After returning:
  ┌─────────────────────────────────────────────────────┐
  │  MySelect is dropped                                │
  │  → taxi future is DROPPED  ← CANCELLATION!         │
  │  → taxi's Drop impl runs                           │
  │  → any resources taxi held are freed                │
  └─────────────────────────────────────────────────────┘
```

### The drop / cancellation problem (preview of Lesson 24)

When select drops the losing future, that future is **cancelled mid-execution**. Whatever state it was in — gone. This has real consequences:

- If the future had written half a message to a buffer — that buffer is now incomplete
- If the future had acquired a lock — the lock guard is dropped (unlocked), but any partial work under the lock is lost
- If the future was a network request — the request is abandoned; the server may still process it

```
Cancelled future's lifecycle:

  poll #1: started work, allocated buffer        ┐
  poll #2: filled half the buffer                │  all of this
  poll #3: about to finish...                    │  state is LOST
           ↓                                     │
  DROP: future is destroyed, buffer freed        ┘
```

This is **cancellation safety** — a topic we will cover in depth in Lesson 24. For now, remember: **select drops the loser, and the loser may have done partial work**.

Rule of thumb: a future is cancellation-safe if dropping it at any await point does not lose data or leave things in an inconsistent state.

## FuturesUnordered (brief mention)

What if you have not 2 but 100 futures, and you want to process results as they complete? `FuturesUnordered` from the `futures` crate is a collection that polls all contained futures and yields results in completion order.

```rust
use futures::stream::FuturesUnordered;
use futures::StreamExt;

let mut futs = FuturesUnordered::new();
futs.push(fetch("url1"));
futs.push(fetch("url2"));
futs.push(fetch("url3"));

while let Some(result) = futs.next().await {
    println!("Got: {result:?}");
}
```

Think of it as a dynamic `select` over many futures. We will use it in later lessons.

## Exercises

### Exercise 1: MyJoin

Implement `MyJoin<A, B>` — a future that polls two sub-futures and returns `(A::Output, B::Output)` when both are done. Implement the `Future` trait by hand.

Hints:
- Store each sub-future and an `Option` for each result
- On each poll, poll any sub-future whose result is still `None`
- Return `Ready` only when both `Option`s are `Some`
- Remember to call `cx.waker().wake_by_ref()` when returning `Pending`

### Exercise 2: MySelect

Implement `MySelect<A, B>` — a future that polls two sub-futures and returns whichever completes first. Use an enum `Either<L, R>` for the return type.

Make both sub-futures `NamedFuture` structs that **print a message when dropped**. Run the demo and observe the loser printing its drop message.

### Exercise 3: join_all for a Vec

Implement a `JoinAll<F>` future that takes a `Vec<F>` and returns `Vec<F::Output>`. This generalizes `MyJoin` from 2 futures to N futures.

Hints:
- Store `Vec<Option<F>>` for the futures and `Vec<Option<F::Output>>` for results
- On each poll, iterate and poll any future that is still `Some`
- When a future completes, store its result and replace the future with `None`
- Return `Ready` when all results are `Some`

### Exercise 4: map combinator

Implement a `Map<F, Func>` future that wraps a future `F` and applies a function `Func` to its output when ready. This lets you write:

```rust
let doubled = Map::new(CountdownFuture { count: 3 }, |()| 42);
// polls the countdown, then applies the function → returns 42
```

Hints:
- Store the inner future and an `Option<Func>` (take the function out on Ready)
- On poll: poll the inner future. If Ready, apply the function and return the mapped result. If Pending, return Pending.
