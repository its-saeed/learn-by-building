# Lesson 12: Replay Attack Defense

> **Alice's Bookstore — Chapter 12**
>
> Alice's encrypted, authenticated bookstore is running smoothly. Then she notices something strange in her logs: a customer named Dave "bought" the same book 47 times in one minute. Dave calls:
>
> *"I only clicked 'Buy' once! But I got charged 47 times!"*
>
> Bob investigates: *"Mallory recorded the encrypted 'buy book' message from the network. She can't read or modify it — your encryption and authentication are solid. But she can REPLAY it. She sent the exact same encrypted bytes 46 more times, and your server processed each one as a valid purchase."*
>
> *"But the messages are encrypted! How can she reuse them?"*
>
> *"She doesn't need to understand the message. She just copies the bytes and sends them again. Your server sees valid encryption, valid auth, decrypts it, and processes it. You need sequence numbers — so your server can say 'I already processed message #7, this is a duplicate.'"*

## Real-life analogy: the receipt trick

```
Without replay defense:
  You pay for dinner. Waiter gives you a receipt.
  A thief photographs your receipt.
  Next day, thief shows the receipt: "I already paid, here's proof"
  Restaurant accepts it — same valid receipt!

With replay defense (sequence numbers):
  Receipt #001: dinner, $50, 2024-04-07
  Receipt #002: lunch, $20, 2024-04-08
  Restaurant tracks: "I've already processed #001"
  Thief shows #001 again → "Already used. Rejected."
```

## The attack

In Lessons 9 and 10, we use random nonces for each message. This prevents nonce reuse, but it doesn't prevent **replay attacks**.

An attacker records an encrypted message (they don't need to decrypt it). Later, they send the exact same bytes again. The server decrypts it successfully — it's a valid ciphertext with a valid nonce. The server processes the message a second time.

```
Client sends: "transfer $100 to Bob"  (encrypted)
         │
         ├──────► Server processes it ✓ ($100 sent)
         │
Attacker records it, replays later:
         │
         └──────► Server processes it again ✓ ($100 sent AGAIN!)
```

The attacker can't read or modify the message, but they can **repeat** it.

## The defense: sequence numbers

Replace random nonces with a **counter**. Both sides maintain a send counter and a receive counter, starting at 0.

```
Message 0: nonce = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
Message 1: nonce = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
Message 2: nonce = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]
...
```

The receiver expects nonces in order. If it receives a nonce it already saw → **reject** (replay). If it receives an out-of-order nonce → **reject** (reordering attack).

This is exactly what TLS 1.3 does. Each record has an implicit sequence number used as the nonce.

### Why this also prevents nonce reuse

With random nonces, there's a (tiny) chance of collision — two messages get the same random nonce. With a counter, nonces are guaranteed unique as long as the counter doesn't wrap around. A 64-bit counter can handle 2^64 messages — far more than any session will ever send.

## The new message format

```
Before (Lessons 7-8):
  [2B length][12B random nonce][ciphertext + tag]

After (Lesson 10):
  [2B length][ciphertext + tag]
  Nonce is derived from sequence number — not sent on the wire!
```

The nonce is no longer transmitted. Both sides know it because they track the counter independently. This saves 12 bytes per message AND eliminates the possibility of an attacker manipulating the nonce.

## Real-world scenarios

### Banking transaction replay

Alice sends an encrypted bank transfer. Mallory captures the encrypted bytes (she can't decrypt them). A week later, Mallory sends the exact same bytes to the bank. Without replay protection, the bank decrypts it, sees a valid transfer request, and processes it again.

With sequence numbers: the bank expects message #4728 next. Mallory's replayed message was #4500. The bank rejects it — wrong sequence number.

### Game server cheating

In an online game, a player sends "use health potion" (encrypted). An attacker captures this message and replays it 100 times. Without replay protection, the player heals 100 times from one potion.

With sequence numbers: the server expects the next sequence number. Replayed messages are instantly rejected.

### TLS 1.3 record numbers

Every TLS 1.3 record has an implicit 64-bit sequence number:
- Client → Server: client maintains `client_seq = 0, 1, 2, ...`
- Server → Client: server maintains `server_seq = 0, 1, 2, ...`

The sequence number is XORed with a per-direction IV (derived during key schedule) to produce the nonce:
```
nonce = IV XOR sequence_number
```

This guarantees unique nonces AND prevents replay.

## How to build the counter nonce

```rust
fn counter_nonce(counter: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[4..12].copy_from_slice(&counter.to_be_bytes());
    nonce
}
```

First 4 bytes are zero, last 8 bytes are the big-endian counter. This gives you 2^64 unique nonces.

## Exercises

### Exercise 1: Counter-based encryption (implemented in 12-replay-server.rs and 12-replay-client.rs)
Replace random nonces with counters. Don't send the nonce on the wire — derive it from the sequence number on both sides.

### Exercise 2: Demonstrate the attack
Take the Lesson 12 server (random nonces). Record an encrypted message with `tcpdump`. Replay the raw bytes with a script. Show the server decrypts and processes it again. Then show the Lesson 12 server rejects the replay.

### Exercise 3: Out-of-order detection
Send messages 0, 1, 2, then replay message 1. The receiver should reject it because it already processed sequence 1 and expects sequence 3.

### Exercise 4: Sliding window (advanced)
In UDP protocols (like DTLS), messages can arrive out of order legitimately. Implement a sliding window: accept messages within a window of N sequence numbers, reject anything older. This is how DTLS and IPsec handle replay protection over unreliable transport.
