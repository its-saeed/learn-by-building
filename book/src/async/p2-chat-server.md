# Project 2: Multi-threaded Chat Server

Build a fully working chat server on top of the async runtime you built in
Lessons 8-14. No tokio, no async-std -- just your reactor, executor, channels,
and timers. This project proves your runtime can handle real concurrent I/O.

## What you'll build

A TCP chat server where:
- Multiple clients connect via `telnet` or `nc`
- Messages from any client are broadcast to all other connected clients
- Each client has a nickname (default: `user-N`), changeable with `/nick name`
- The server detects disconnects (EOF or broken pipe) and announces departures
- The server runs on your work-stealing runtime across multiple threads

### Feature list

- **Accept loop** -- `AsyncTcpListener` accepts connections and spawns a task
  per client
- **Broadcast** -- an mpsc channel per client; incoming messages fan out to
  every other client's channel
- **Commands** -- `/nick <name>` changes display name, `/who` lists connected
  users, `/quit` disconnects
- **Disconnect detection** -- read returning 0 bytes or an error triggers
  cleanup and a "user left" broadcast
- **Graceful shutdown** -- Ctrl-C sets a flag; the accept loop exits and all
  client tasks drain

## Key concepts

- **Shared state** -- a `HashMap<ClientId, ClientHandle>` behind an async-aware
  mutex or accessed from a dedicated broker task
- **Broker pattern** -- one task owns the client map and receives events
  (join / leave / message) over a channel, avoiding shared mutable state
- **Backpressure** -- bounded per-client channels prevent a slow reader from
  exhausting memory
- **Cancellation** -- when a client disconnects, its task is dropped; select
  ensures no leaked futures
- **Testing** -- spawn the server in a background task, connect with multiple
  `AsyncTcpStream` clients from test tasks, and assert message delivery

## Exercises

1. **Basic chat** -- implement the accept loop, per-client read loop, and
   broadcast. Connect two `nc` sessions and verify messages flow both ways.

2. **Commands and nicks** -- add `/nick`, `/who`, and `/quit`. Verify that
   broadcast messages show the updated nickname after a `/nick` change.

3. **Load test** -- spawn 100 client tasks that each send 10 messages. Assert
   every client receives all 900 messages from others (100 clients x 10
   messages - own 10). Measure total time on your work-stealing runtime.
