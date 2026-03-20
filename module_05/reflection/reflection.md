# Part C — Reflection: Immutability and Concurrency in Clojure and Rust

## 1. What does "update" mean in each solution?

### Clojure

In Clojure there is no such thing as updating a value — there is only
producing a *new* value that differs from the old one.  When we call:

```clojure
(defn append-event [store event]
  (update store :events conj event))
```

`update` calls `conj` on the current `:events` vector, which returns a brand-new
persistent vector.  Then `update` wraps it in a new map.  The symbol `store` still
refers to the old map; nothing has been modified.  The caller's binding is
unchanged, and any other thread holding a reference to the same map is unaffected.

This is the default in Clojure: every data structure is immutable unless you
explicitly reach for a reference type (`atom`, `ref`, `agent`).  In the
concurrency demo we use exactly one `atom` — not to mutate the store, but to
*publish* a pointer to the latest immutable store version.  The atom is the
smallest possible mutable surface.

### Rust

Rust's ownership system enforces a different but equally strong guarantee:
`append_event` takes `&self` (shared reference, no mutation rights) and returns
a new `Store` by value.  If you want to mutate you must declare `&mut self`, and
the borrow checker ensures no other reference exists at the same time.

```rust
pub fn append_event(&self, event: Event) -> Store {
    let mut new_events: Vec<Event> = (*self.events).clone();  // explicit copy
    new_events.push(event);
    Store { events: Arc::new(new_events) }
}
```

The cost is explicit: we call `.clone()` on the Vec.  Clojure hides the same
concept behind structural sharing, making it feel "free".  In Rust you can see
every allocation.  This is both a strength (nothing is hidden from the
optimizer) and a friction point (you must think about it).

**Key difference**: Clojure's "update" is *structural* — produce a new root
node that shares most of the tree with the old version.  Rust's "update" here
is a *full copy* of the Vec — O(n) per append.  The `CheckpointStore` and
`CachingStore` variants reduce the replay cost but do not recover the O(log n)
memory-sharing that Clojure's persistent vectors provide natively.  A Rust
crate such as `im` (immutable collections) would close that gap.

---

## 2. What data is shared between versions?

### Clojure

Clojure vectors are 32-ary tries.  A 1000-element vector has depth 2; `conj`
allocates one new node at each level of the path to the new leaf — about
32 words — and reuses every other node.  So versions share roughly
`(N - log₃₂ N)` nodes.  The garbage collector reclaims unreachable versions
automatically.

In our store, two consecutive versions look like:

```
s4 {:events PVec@0x1234}
s5 {:events PVec@0x5678}   ← new root, tail pointer
                    ↘
              shared interior nodes from PVec@0x1234
```

### Rust

With `Arc<Vec<Event>>`, two versions share **nothing** in the Vec itself — each
version owns its own heap-allocated Vec.  What they share is the *Arc control
block* until the last clone is dropped.

```
s4.events → Arc(Vec[e0..e3], rc=1)  ← will be freed when s4 goes out of scope
s5.events → Arc(Vec[e0..e4], rc=1)  ← own allocation
```

In the concurrency demo, multiple threads may hold `Arc<Store>` clones pointing
to the same Vec.  Each thread's Arc bumps the reference count; the Vec is freed
only when all clones drop.  This is safe reference-counted sharing, not
structural sharing.

The `CheckpointStore` shares `Arc<Vec<(usize, State)>>` for the checkpoint list
between consecutive versions when no new checkpoint was triggered, which is a
small optimisation — but the event Vec itself is always fully copied.

---

## 3. Why are readers easy to reason about?

In both languages, once a reader has a handle to a value (a Clojure map or a
Rust `Arc<Store>`), that value is frozen forever.  No amount of concurrent
writing can change it.  This gives us three properties:

1. **No torn reads.**  A reader cannot observe a half-written state because
   there is no writing happening to the data it holds.  In Clojure this is
   structural — maps are values.  In Rust this is enforced by the borrow
   checker: `&self` is shared, `&mut self` is exclusive.

2. **No lock needed during computation.**  Both concurrency demos acquire
   a lock for *one microsecond* — just long enough to copy a pointer.  The
   actual work (fold over events, build a HashMap) runs lock-free.

3. **Compositional reasoning.**  `snapshot_at(store, v)` is a pure function.
   Its result depends only on its inputs.  You can call it from any thread,
   at any time, as many times as you like; the result is always the same.
   Unit tests on pure functions are valid proofs of concurrent behaviour.

The contrast with mutable shared state is stark.  If `State` were a
`Mutex<HashMap>` that the writer updated in-place, every reader would need to
hold the lock for the duration of its traversal, serialising all concurrent
accesses.  Immutability eliminates the need for reader locks entirely.

---

## 4. Tradeoffs

| Dimension | Clojure | Rust (plain Store) | Rust (CheckpointStore) |
|---|---|---|---|
| **Append cost** | O(log₃₂ N) amortised | O(N) — full Vec clone | O(N) for Vec; O(M) per checkpoint |
| **Memory per version** | O(log N) new nodes | O(N) new Vec | O(N) new Vec + checkpoint State |
| **snapshot_at cost** | O(v) replay from v=0 | O(v) replay | O(v − nearest_checkpoint) |
| **Structural sharing** | Automatic (trie) | None by default | None (Vec); checkpoints shared via Arc |
| **GC pressure** | JVM GC handles old versions | Drop / RAII (deterministic) | Drop / RAII |
| **Concurrency model** | Atom (CAS) + immutable values | Arc + RwLock for pointer swap | Same as plain Store |
| **Ergonomics** | `conj` / `assoc` look like mutation | `.clone()` forces awareness | More boilerplate |
| **Safety guarantees** | Dynamic; violations possible at runtime in theory (agents can fail) | Compile-time; borrow checker rejects invalid sharing | Same as plain Store |

### Memory

Every appended version keeps the entire previous event log alive.  For a log
of N events with V versions, the naïve Rust approach uses O(N²) total memory
(1 + 2 + … + N words).  Clojure uses O(N log N) due to structural sharing.
In practice you would not keep all versions in memory; an event-sourced system
typically keeps only the current store reference (or a small window of recent
versions) and discards older ones.

### Performance

The O(v) replay on every `snapshot_at` call is the primary performance
concern.  Both languages benefit from checkpointing (store a pre-computed
snapshot every K events; replay only the tail).  Clojure's `memoize` is a
one-liner solution; Rust needs a `HashMap<usize, State>` and a lock, which the
`CachingStore` provides.

### The interior-mutability trade-off (`CachingStore`)

Adding a `Mutex<HashMap<usize, State>>` cache to `CachingStore` restores O(1)
repeated calls to `snapshot_at(v)` while keeping the external API pure.  But
it changes the reasoning model in a subtle way:

- `snapshot_at` now has a hidden *write* side-effect (cache fill).  Two threads
  racing on the same version will both compute the snapshot and one will "win"
  the cache write — the correct value is still returned, but work is doubled
  on the first call.
- Lock granularity matters: if the cache computation itself ever tried to call
  back into a locked resource, deadlock would be possible.  Here it does not,
  but the invariant must be maintained as the code evolves.
- Profiling and testing become harder because `snapshot_at` is no longer
  idempotent in terms of allocations.

The Clojure `make-checkpoint-store` analogue uses an atom for the same purpose.
The atom is safer in one sense — its swap! is lock-free (compare-and-swap) —
but it shares the same reasoning cost: the function now has a side-effect that
is invisible to callers.

**Conclusion**: interior mutability should be confined to caching and I/O, never
to domain logic.  The `CachingStore` is safe because its mutation is *additive
only* (we never invalidate or overwrite a cached entry) and the cached value is
always recomputed from a deterministic pure function.  That narrow contract is
what makes it defensible.

---

## Summary

Both solutions demonstrate the same core insight: **immutability trades write
throughput for read simplicity**.  Appending a new event is more expensive than
mutating a field in place, but every reader of every version is free from locks,
races, and partial reads — permanently, by construction.

Clojure makes immutability the path of least resistance and provides structural
sharing as a built-in efficiency.  Rust makes immutability explicit and visible,
demanding you account for every copy, but in return offers compile-time proof
that nothing was mutated behind your back.  The two languages reach the same
destination from opposite directions: Clojure by making mutation an opt-in, Rust
by making sharing an opt-in.
