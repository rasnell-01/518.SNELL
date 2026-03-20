Objectives

Experience “update” as value transformation (old versions remain valid).
Observe structural sharing (Clojure persistent collections; Rust via Arc + persistent-ish design).
See implications for concurrency: readers don’t need locks if data never changes.
Contrast: Clojure’s default immutability vs Rust’s immutability-by-default + explicit interior mutability when needed.
Problem statement

Build a tiny event-sourced system for a todo appThe system stores an append-only log of events and can produce snapshots of state at any version.

Domain

Events

AddItem {id, text}
CompleteItem {id}
RenameItem {id, new_text}
DeleteItem {id}
State:

Map from id -> {text, completed?}
You must support:

append-event

Input: current store + new event
Output: new store with event appended (old store still usable)
snapshot-at

Input: store + version number v
Output: state after applying first v events
history

Return a list of all snapshots or selected snapshots (e.g., every N events)
time travel demo

Show that after “updating,” you can still query the earlier version and get the earlier answer.
Required deliverables

Part A — Core implementation (both languages)

append_event(store, event) -> store'

snapshot_at(store, v) -> state

current_snapshot(store) -> state

demo() that:

builds store versions s0, s1, s2, ...
prints (or asserts) states at multiple versions
proves old versions are unchanged
Part B — Concurrency experiment (both languages)

Spin up:

1 writer thread/task producing versions s0..sN
Multiple readers repeatedly calling snapshot_at on older versions
Goal: demonstrate the design where readers never block and never see torn/partial updates.

Clojure: use a plain immutable value flow, plus optionally show atom just to publish the “latest version” safely (publishing is separate from mutating the store).
Rust: publish Arc<Store> versions; readers clone Arc and snapshot safely.
Part C — Short reflection (1–2 pages)

Address:

What does “update” mean in each solution?
What data is shared between versions?
Why are readers easy to reason about?
What tradeoffs appear (memory, performance, ergonomics)?
Hints and Suggestions

Clojure guidance

Represent store as:

{:events [...]} (vector of events)
Snapshots:

reduce apply-event initial-state (subvec events 0 v)
Key immutability “aha”:

conj on a vector returns a new vector, old one remains.
Persistent vectors/maps provide structural sharing, so versions are cheap.
Stretch: memoize snapshots every K events for faster snapshot_at without mutation.

Rust guidance

Rust doesn’t ship a persistent vector/map in std, so make the immutability story explicit.

Recommended representation:

Store { events: Arc<Vec<Event>> }
append_event clones the underlying Vec (costly but clear), then wraps in Arc.
Then extend to a more “persistent” feel:

Keep Arc<[Event]> (boxed slice) or Arc<Vec<Event>>

Add checkpoints:

Store { events: Arc<Vec<Event>>, checkpoints: Arc<Vec<(usize, State)>> }
Every K appends, compute snapshot and store it (still immutable: new Arc<Vec<_>>).
Concurrency:

Share versions with Arc<Store>
Readers clone Arc and compute snapshots with no locks.
Stretch (Extra Credit): Introduce optional interior mutability only for caching (e.g., OnceLock or Mutex<HashMap<usize, State>>) and justify in your reflection why that changes the reasoning model.

Testing requirements (both languages)

Write tests that prove immutability properties:

Old version unchanged

Build s0, then s1 = append(s0, e1)
Assert: snapshot_at(s0, 0) equals initial state
Assert: snapshot_at(s1, 1) includes e1
Non-interference

s2 = append(s1, e2)
Assert: snapshot_at(s1, 1) still same as before creating s2
Concurrency sanity

Readers repeatedly query earlier versions while writer produces later ones
Assert invariants never violated (no panics; snapshot results stable)
Grading rubric (100 pts)

Correctness of event application + snapshots (35)
Versioning / time travel demonstrated clearly (20)
Concurrency experiment implemented and explained (20)
Tests (15)
Reflection quality (10)
