//! # todo-store — immutable, event-sourced todo list
//!
//! Core design:
//!   Store { events: Arc<Vec<Event>> }
//!
//! `append_event` clones the underlying Vec and wraps it in a *new* Arc.
//! The caller keeps its original Arc pointing to the old Vec — no mutation,
//! no unsafe, no lock.  Concurrency falls out for free: share an Arc<Store>
//! and readers never block.
//!
//! The explicit clone makes the O(n) cost visible.  The `CheckpointStore`
//! variant amortises it with periodic snapshots.

use std::collections::HashMap;
use std::sync::Arc;

// ─────────────────────────────────────────────────────────────────────────
// Domain types
// ─────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    AddItem    { id: u64, text: String },
    CompleteItem { id: u64 },
    RenameItem { id: u64, new_text: String },
    DeleteItem { id: u64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub text:      String,
    pub completed: bool,
}

/// A snapshot of the todo list at a given version.
pub type State = HashMap<u64, Item>;

// ─────────────────────────────────────────────────────────────────────────
// State machine
// ─────────────────────────────────────────────────────────────────────────

/// Fold a single event onto `state`.  Takes ownership of `state` so the
/// compiler can re-use the allocation (fold hands ownership back next iter).
pub fn apply_event(mut state: State, event: &Event) -> State {
    match event {
        Event::AddItem { id, text } => {
            state.insert(*id, Item { text: text.clone(), completed: false });
        }
        Event::CompleteItem { id } => {
            if let Some(item) = state.get_mut(id) {
                item.completed = true;
            }
        }
        Event::RenameItem { id, new_text } => {
            if let Some(item) = state.get_mut(id) {
                item.text = new_text.clone();
            }
        }
        Event::DeleteItem { id } => {
            state.remove(id);
        }
    }
    state
}

fn replay(events: &[Event]) -> State {
    events.iter().fold(State::new(), apply_event)
}

// ─────────────────────────────────────────────────────────────────────────
// Store — Part A core
// ─────────────────────────────────────────────────────────────────────────

/// Immutable event store.
///
/// `events` is behind an `Arc` so that cloning a `Store` is O(1) (pointer
/// copy + atomic ref-count bump).  `append_event` builds a *new* Vec, wraps
/// it in a new Arc, and returns a new Store.  The old Arc — and the old Vec
/// it owns — is untouched.
#[derive(Debug, Clone)]
pub struct Store {
    events: Arc<Vec<Event>>,
}

impl Default for Store {
    fn default() -> Self { Self::new() }
}

impl Store {
    pub fn new() -> Self {
        Store { events: Arc::new(Vec::new()) }
    }

    /// Returns a **new** `Store` with `event` appended.
    ///
    /// The original `self` is unchanged (Rust's ownership guarantees this at
    /// compile time — if you tried to mutate `self.events` in place you would
    /// need `&mut self`, which the caller would have to hand over explicitly).
    pub fn append_event(&self, event: Event) -> Store {
        let mut new_events: Vec<Event> = (*self.events).clone();   // O(n)
        new_events.push(event);
        Store { events: Arc::new(new_events) }
    }

    pub fn len(&self) -> usize { self.events.len() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }

    /// Replay the first `v` events and return the resulting state.
    pub fn snapshot_at(&self, v: usize) -> State {
        assert!(v <= self.events.len(),
                "v={v} out of range (store has {} events)", self.events.len());
        replay(&self.events[..v])
    }

    pub fn current_snapshot(&self) -> State {
        self.snapshot_at(self.events.len())
    }

    /// All snapshots: version 0 … N.
    pub fn history(&self) -> Vec<State> {
        (0..=self.events.len())
            .map(|v| self.snapshot_at(v))
            .collect()
    }

    /// Snapshots at versions 0, n, 2n, …
    pub fn history_every_n(&self, n: usize) -> Vec<(usize, State)> {
        let len = self.events.len();
        (0..=len).step_by(n.max(1))
                 .map(|v| (v, self.snapshot_at(v)))
                 .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// CheckpointStore — Stretch: faster snapshot_at via stored checkpoints
//
// Every `checkpoint_interval` appends we compute and cache a snapshot.
// Both `events` and `checkpoints` are behind Arc<Vec<_>>, so all prior
// versions still refer to their own Arcs; appending a checkpoint never
// invalidates an older CheckpointStore.
// ─────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CheckpointStore {
    /// The full event log.
    pub events: Arc<Vec<Event>>,
    /// Monotonically growing list of (version, state) checkpoints.
    pub checkpoints: Arc<Vec<(usize, State)>>,
    pub checkpoint_interval: usize,
}

impl CheckpointStore {
    pub fn new(checkpoint_interval: usize) -> Self {
        CheckpointStore {
            events:              Arc::new(Vec::new()),
            checkpoints:         Arc::new(Vec::new()),
            checkpoint_interval: checkpoint_interval.max(1),
        }
    }

    pub fn append_event(&self, event: Event) -> CheckpointStore {
        // Clone the event log (O(n)) and push the new event.
        let mut new_events: Vec<Event> = (*self.events).clone();
        new_events.push(event);
        let new_len = new_events.len();

        // If we've hit a checkpoint boundary, compute and store the snapshot.
        let new_checkpoints: Arc<Vec<(usize, State)>> =
            if new_len % self.checkpoint_interval == 0 {
                let snap = replay(&new_events);
                let mut cps: Vec<(usize, State)> = (*self.checkpoints).clone();
                cps.push((new_len, snap));
                Arc::new(cps)
            } else {
                Arc::clone(&self.checkpoints)
            };

        CheckpointStore {
            events:              Arc::new(new_events),
            checkpoints:         new_checkpoints,
            checkpoint_interval: self.checkpoint_interval,
        }
    }

    pub fn len(&self) -> usize { self.events.len() }

    /// Snapshot at version `v`, using the nearest checkpoint to minimise work.
    pub fn snapshot_at(&self, v: usize) -> State {
        assert!(v <= self.events.len());

        // Find the latest checkpoint whose version is ≤ v.
        let best = self.checkpoints
            .iter()
            .rev()
            .find(|(cp_v, _)| *cp_v <= v);

        match best {
            Some((cp_v, cp_state)) =>
                // Replay only the events *after* the checkpoint.
                self.events[*cp_v..v]
                    .iter()
                    .fold(cp_state.clone(), apply_event),
            None =>
                replay(&self.events[..v]),
        }
    }

    pub fn current_snapshot(&self) -> State {
        self.snapshot_at(self.events.len())
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Extra-credit: interior mutability for lazy caching with OnceLock
//
// `CachingStore` wraps a `Store` and uses a `Mutex<HashMap>` to cache
// snapshot results on first access.  The Mutex is an *implementation detail*
// — callers still see `snapshot_at(&self, v)` (immutable reference).
//
// Why this changes the reasoning model:
//   - With a plain Store, snapshot_at is pure: no side effects, freely
//     parallelisable, no ordering constraints.
//   - With CachingStore, snapshot_at has a hidden write side-effect (cache
//     fill).  Two threads calling snapshot_at(v) concurrently will contend
//     on the Mutex for that one computation, then one wins and caches.
//   - The *external* result is still deterministic (same value every time),
//     so callers see referential transparency — but you must now think about
//     lock granularity and potential deadlocks if the computation itself
//     tried to lock the same mutex (it doesn't here, but the risk exists).
// ─────────────────────────────────────────────────────────────────────────

use std::sync::Mutex;

#[derive(Debug)]
pub struct CachingStore {
    inner: Store,
    cache: Mutex<HashMap<usize, State>>,
}

impl CachingStore {
    pub fn new(store: Store) -> Self {
        CachingStore { inner: store, cache: Mutex::new(HashMap::new()) }
    }

    /// Append returns a new CachingStore wrapping the new inner Store.
    /// The cache is intentionally *not* carried over (it belongs to the old
    /// version; the new version will lazily build its own).
    pub fn append_event(&self, event: Event) -> CachingStore {
        CachingStore::new(self.inner.append_event(event))
    }

    pub fn snapshot_at(&self, v: usize) -> State {
        // Fast path: already cached.
        {
            let cache = self.cache.lock().unwrap();
            if let Some(snap) = cache.get(&v) {
                return snap.clone();
            }
        }
        // Slow path: compute, then cache.
        let snap = self.inner.snapshot_at(v);
        self.cache.lock().unwrap().insert(v, snap.clone());
        snap
    }

    pub fn len(&self) -> usize { self.inner.len() }
}

// ─────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::Duration;

    // ── helpers ────────────────────────────────────────────────────────

    fn add(id: u64, text: &str) -> Event {
        Event::AddItem { id, text: text.to_string() }
    }
    fn complete(id: u64) -> Event { Event::CompleteItem { id } }
    fn rename(id: u64, new_text: &str) -> Event {
        Event::RenameItem { id, new_text: new_text.to_string() }
    }
    fn delete(id: u64) -> Event { Event::DeleteItem { id } }

    // ── 1. append_event immutability ───────────────────────────────────

    #[test]
    fn old_store_unchanged_after_append() {
        let s0 = Store::new();
        let s1 = s0.append_event(add(1, "Buy milk"));
        let _s2 = s1.append_event(complete(1));

        // s0 still has 0 events
        assert_eq!(s0.len(), 0);
        // s1 still has 1 event
        assert_eq!(s1.len(), 1);
        // s1 snapshot at v=1 is unaffected by creating s2
        let snap = s1.snapshot_at(1);
        assert_eq!(snap[&1].text, "Buy milk");
        assert!(!snap[&1].completed);
    }

    // ── 2. snapshot_at correctness ─────────────────────────────────────

    #[test]
    fn snapshot_at_v0_is_empty() {
        let store = Store::new()
            .append_event(add(1, "A"));
        assert!(store.snapshot_at(0).is_empty());
    }

    #[test]
    fn snapshot_at_all_event_types() {
        let store = Store::new()
            .append_event(add(1, "Buy milk"))       // v1
            .append_event(add(2, "Write code"))     // v2
            .append_event(complete(1))              // v3
            .append_event(rename(2, "Write tests")) // v4
            .append_event(delete(1));               // v5

        // v0 — empty
        assert!(store.snapshot_at(0).is_empty());

        // v1 — only item 1
        let s1 = store.snapshot_at(1);
        assert_eq!(s1.len(), 1);
        assert_eq!(s1[&1].text, "Buy milk");
        assert!(!s1[&1].completed);

        // v3 — item 1 completed
        let s3 = store.snapshot_at(3);
        assert!(s3[&1].completed);
        assert_eq!(s3[&2].text, "Write code");

        // v4 — item 2 renamed
        assert_eq!(store.snapshot_at(4)[&2].text, "Write tests");

        // v5 — item 1 deleted
        let s5 = store.snapshot_at(5);
        assert!(!s5.contains_key(&1));
        assert!(s5.contains_key(&2));
    }

    // ── 3. Time travel / non-interference ─────────────────────────────

    #[test]
    fn time_travel_non_interference() {
        let s1 = Store::new().append_event(add(1, "Task"));
        let snap_before = s1.snapshot_at(1);

        // Build s2 — must not affect s1
        let s2 = s1.append_event(complete(1));
        let snap_after = s1.snapshot_at(1);

        assert_eq!(snap_before, snap_after);
        assert!(!snap_before[&1].completed);
        assert!(s2.snapshot_at(2)[&1].completed);

        // Older version sees only its own events
        let s0 = Store::new();
        let s3 = s0.append_event(add(1, "A"))
                   .append_event(add(2, "B"))
                   .append_event(complete(1));

        // s0 is untouched
        assert!(s0.current_snapshot().is_empty());
        // s3 sees everything
        assert!(s3.snapshot_at(3)[&1].completed);
    }

    // ── 4. history ──────────────────────────────────────────────────────

    #[test]
    fn history_has_n_plus_one_entries() {
        let store = (1..=5).fold(Store::new(), |s, i| s.append_event(add(i, "x")));
        assert_eq!(store.history().len(), 6); // versions 0..5
    }

    #[test]
    fn history_every_n_correct_versions() {
        let store = (1..=10).fold(Store::new(), |s, i| s.append_event(add(i, "x")));
        let h = store.history_every_n(3);
        let versions: Vec<usize> = h.iter().map(|(v, _)| *v).collect();
        assert_eq!(versions, vec![0, 3, 6, 9]);
    }

    // ── 5. CheckpointStore ─────────────────────────────────────────────

    #[test]
    fn checkpoint_store_agrees_with_plain_store() {
        let events = vec![
            add(1, "A"), add(2, "B"), complete(1), rename(2, "B2"), delete(1),
        ];
        let plain: Store = events.iter().cloned()
            .fold(Store::new(), |s, e| s.append_event(e));
        let cp: CheckpointStore = events.into_iter()
            .fold(CheckpointStore::new(2), |s, e| s.append_event(e));

        for v in 0..=plain.len() {
            assert_eq!(plain.snapshot_at(v), cp.snapshot_at(v),
                       "Mismatch at v={v}");
        }
    }

    // ── 6. CachingStore (extra credit) ─────────────────────────────────

    #[test]
    fn caching_store_returns_same_result_twice() {
        let store = CachingStore::new(
            Store::new()
                .append_event(add(1, "A"))
                .append_event(complete(1))
        );
        let first  = store.snapshot_at(2);
        let second = store.snapshot_at(2);
        assert_eq!(first, second);
        assert!(second[&1].completed);
    }

    // ── 7. Concurrency sanity ──────────────────────────────────────────
    //
    // One writer publishes Arc<Store> versions into an RwLock.
    // Multiple readers clone the Arc and call snapshot_at — no other lock.
    // Invariant: every item in every snapshot has a non-empty text.

    #[test]
    fn concurrent_readers_never_see_torn_state() {
        // `latest` holds a pointer to the most-recently-published Store.
        // Readers clone the Arc (O(1)) and then work entirely on their own copy.
        let latest: Arc<RwLock<Arc<Store>>> =
            Arc::new(RwLock::new(Arc::new(Store::new())));

        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        // ── Writer ────────────────────────────────────────────────────
        let latest_w = Arc::clone(&latest);
        let writer = thread::spawn(move || {
            for i in 0..20u64 {
                let new_store = {
                    let cur = latest_w.read().unwrap();
                    Arc::new(cur.append_event(add(i + 1, &format!("item-{i}"))))
                };
                *latest_w.write().unwrap() = new_store;
                thread::sleep(Duration::from_millis(5));
            }
        });

        // ── Readers ───────────────────────────────────────────────────
        let mut handles = vec![];
        for _ in 0..5 {
            let latest_r  = Arc::clone(&latest);
            let errors_r  = Arc::clone(&errors);
            handles.push(thread::spawn(move || {
                for _ in 0..50 {
                    // Grab a pointer to the current store — O(1), brief lock.
                    let store: Arc<Store> =
                        Arc::clone(&*latest_r.read().unwrap());
                    // All further work is lock-free.
                    let v    = store.len();
                    let snap = store.snapshot_at(v);
                    for (id, item) in &snap {
                        if item.text.is_empty() {
                            errors_r.lock().unwrap()
                                    .push(format!("id={id} has empty text"));
                        }
                    }
                    thread::sleep(Duration::from_millis(2));
                }
            }));
        }

        writer.join().unwrap();
        for h in handles { h.join().unwrap(); }

        assert!(errors.lock().unwrap().is_empty(),
                "Errors: {:?}", errors.lock().unwrap());
    }
}
