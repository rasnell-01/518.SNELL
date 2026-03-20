//! todo-store — immutable, event-sourced todo list
//!
//! Single-file layout: types → state machine → Store → CheckpointStore →
//! CachingStore → Part A demo → Part B demo → tests.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

// ═════════════════════════════════════════════════════════════════════════
// Domain types
// ═════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    AddItem     { id: u64, text: String },
    CompleteItem { id: u64 },
    RenameItem  { id: u64, new_text: String },
    DeleteItem  { id: u64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub text:      String,
    pub completed: bool,
}

pub type State = HashMap<u64, Item>;

// ═════════════════════════════════════════════════════════════════════════
// State machine
// ═════════════════════════════════════════════════════════════════════════

pub fn apply_event(mut state: State, event: &Event) -> State {
    match event {
        Event::AddItem { id, text } => {
            state.insert(*id, Item { text: text.clone(), completed: false });
        }
        Event::CompleteItem { id } => {
            if let Some(item) = state.get_mut(id) { item.completed = true; }
        }
        Event::RenameItem { id, new_text } => {
            if let Some(item) = state.get_mut(id) { item.text = new_text.clone(); }
        }
        Event::DeleteItem { id } => { state.remove(id); }
    }
    state
}

fn replay(events: &[Event]) -> State {
    events.iter().fold(State::new(), apply_event)
}

// ═════════════════════════════════════════════════════════════════════════
// Store — core immutable event log
//
// append_event takes &self and returns a NEW Store.
// The old Store is untouched — Rust's type system proves this at compile
// time: &self is a shared reference; mutation would require &mut self.
// ═════════════════════════════════════════════════════════════════════════

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

    /// Returns a brand-new Store with `event` appended.  O(n) clone.
    pub fn append_event(&self, event: Event) -> Store {
        let mut v: Vec<Event> = (*self.events).clone();
        v.push(event);
        Store { events: Arc::new(v) }
    }

    pub fn len(&self) -> usize { self.events.len() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }

    pub fn snapshot_at(&self, v: usize) -> State {
        assert!(v <= self.events.len(),
                "v={v} out of range (store has {} events)", self.events.len());
        replay(&self.events[..v])
    }

    pub fn current_snapshot(&self) -> State {
        self.snapshot_at(self.events.len())
    }

    pub fn history(&self) -> Vec<State> {
        (0..=self.events.len()).map(|v| self.snapshot_at(v)).collect()
    }

    /// Snapshots at versions 0, n, 2n, …
    pub fn history_every_n(&self, n: usize) -> Vec<(usize, State)> {
        let len = self.events.len();
        (0..=len).step_by(n.max(1))
                 .map(|v| (v, self.snapshot_at(v)))
                 .collect()
    }
}

// ═════════════════════════════════════════════════════════════════════════
// CheckpointStore — faster snapshot_at via periodic cached snapshots
// ═════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct CheckpointStore {
    pub events:              Arc<Vec<Event>>,
    pub checkpoints:         Arc<Vec<(usize, State)>>,
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
        let mut new_events: Vec<Event> = (*self.events).clone();
        new_events.push(event);
        let new_len = new_events.len();

        let new_checkpoints = if new_len % self.checkpoint_interval == 0 {
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

    pub fn snapshot_at(&self, v: usize) -> State {
        assert!(v <= self.events.len());
        let best = self.checkpoints.iter().rev().find(|(cp_v, _)| *cp_v <= v);
        match best {
            Some((cp_v, cp_state)) =>
                self.events[*cp_v..v].iter().fold(cp_state.clone(), apply_event),
            None =>
                replay(&self.events[..v]),
        }
    }

    pub fn current_snapshot(&self) -> State {
        self.snapshot_at(self.events.len())
    }
}

// ═════════════════════════════════════════════════════════════════════════
// CachingStore — extra credit: interior mutability for lazy snapshot cache
//
// snapshot_at is externally pure (same inputs → same output) but internally
// writes to a Mutex<HashMap> on cache miss.  See reflection.md for why this
// changes the reasoning model.
// ═════════════════════════════════════════════════════════════════════════

pub struct CachingStore {
    inner: Store,
    cache: Mutex<HashMap<usize, State>>,
}

impl CachingStore {
    pub fn new(store: Store) -> Self {
        CachingStore { inner: store, cache: Mutex::new(HashMap::new()) }
    }

    pub fn append_event(&self, event: Event) -> CachingStore {
        CachingStore::new(self.inner.append_event(event))
    }

    pub fn snapshot_at(&self, v: usize) -> State {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(snap) = cache.get(&v) { return snap.clone(); }
        }
        let snap = self.inner.snapshot_at(v);
        self.cache.lock().unwrap().insert(v, snap.clone());
        snap
    }

    pub fn len(&self) -> usize { self.inner.len() }
}

// ═════════════════════════════════════════════════════════════════════════
// Convenience constructors (mirrors Clojure event factory functions)
// ═════════════════════════════════════════════════════════════════════════

fn add(id: u64, text: &str) -> Event {
    Event::AddItem { id, text: text.to_string() }
}
fn complete(id: u64) -> Event { Event::CompleteItem { id } }
fn rename(id: u64, t: &str) -> Event {
    Event::RenameItem { id, new_text: t.to_string() }
}
fn delete(id: u64) -> Event { Event::DeleteItem { id } }

// ═════════════════════════════════════════════════════════════════════════
// Part A — Time Travel Demo
// ═════════════════════════════════════════════════════════════════════════

fn part_a() {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!(  "║         Part A — Time Travel Demo               ║");
    println!(  "╚══════════════════════════════════════════════════╝\n");

    let s0 = Store::new();
    let s1 = s0.append_event(add(1, "Buy milk"));
    let s2 = s1.append_event(add(2, "Write code"));
    let s3 = s2.append_event(complete(1));
    let s4 = s3.append_event(rename(2, "Write better code"));
    let s5 = s4.append_event(delete(1));

    println!("Event counts:  s0={}  s1={}  s2={}  s5={}",
             s0.len(), s1.len(), s2.len(), s5.len());

    println!("\n--- Snapshots via s5 ---");
    for v in 0..=s5.len() {
        println!("  v{v:02} -> {:?}", s5.snapshot_at(v));
    }

    println!("\n--- Proving old stores are UNCHANGED ---");
    println!("  snapshot_at(s0, 0) = {:?}  [expect empty]", s0.snapshot_at(0));
    println!("  snapshot_at(s1, 1) = {:?}", s1.snapshot_at(1));
    println!("  snapshot_at(s2, 2) = {:?}", s2.snapshot_at(2));

    println!("\n--- history_every_n(s5, 2) ---");
    for (v, state) in s5.history_every_n(2) {
        println!("  v{v:02} -> {} item(s)", state.len());
    }

    println!("\n--- CheckpointStore (interval=2) ---");
    let cp = vec![add(1,"A"), add(2,"B"), complete(1), rename(2,"B2"), delete(1)]
        .into_iter()
        .fold(CheckpointStore::new(2), |s, e| s.append_event(e));
    for v in 0..=cp.len() {
        println!("  v{v:02} -> {:?}", cp.snapshot_at(v));
    }

    assert!(s0.snapshot_at(0).is_empty(), "s0 must still be empty");
    assert_eq!(s1.snapshot_at(1).len(), 1, "s1 must still have 1 item");
    println!("\n  Assertions passed — old versions are provably unchanged.");
}

// ═════════════════════════════════════════════════════════════════════════
// Part B — Concurrency Demo
//
//   latest: Arc<RwLock<Arc<Store>>>
//
//   Writer builds new Arc<Store> OUTSIDE the lock, then swaps the pointer
//   under a write lock held for microseconds only.
//
//   Readers clone the Arc under a brief read lock, then do all snapshot
//   work with zero locks — the immutable Vec can never change under them.
// ═════════════════════════════════════════════════════════════════════════

fn part_b() {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!(  "║       Part B — Concurrency Demo                 ║");
    println!(  "╚══════════════════════════════════════════════════╝\n");

    let latest: Arc<RwLock<Arc<Store>>> =
        Arc::new(RwLock::new(Arc::new(Store::new())));

    let errors: Arc<Mutex<Vec<String>>>  = Arc::new(Mutex::new(Vec::new()));
    let samples: Arc<Mutex<Vec<(usize, usize, usize)>>> =
        Arc::new(Mutex::new(Vec::new()));

    // ── Writer ────────────────────────────────────────────────────────
    let writer = {
        let latest = Arc::clone(&latest);
        thread::spawn(move || {
            let events = vec![
                add(1, "Task A"), add(2, "Task B"), complete(1),
                add(3, "Task C"), rename(2, "Task B (renamed)"),
                delete(1), add(4, "Task D"), complete(3),
            ];
            for event in events {
                let new_store: Arc<Store> = {
                    let guard = latest.read().unwrap();
                    Arc::new(guard.append_event(event))
                };
                *latest.write().unwrap() = new_store;
                thread::sleep(Duration::from_millis(15));
            }
            println!("  [writer] published {} events",
                     latest.read().unwrap().len());
        })
    };

    // ── Readers ───────────────────────────────────────────────────────
    let mut handles = vec![];
    for rid in 0..5_usize {
        let latest  = Arc::clone(&latest);
        let errors  = Arc::clone(&errors);
        let samples = Arc::clone(&samples);
        handles.push(thread::spawn(move || {
            for _ in 0..30 {
                // Grab the pointer (brief read lock), then release the lock.
                let store: Arc<Store> = Arc::clone(&*latest.read().unwrap());

                // All work below is lock-free.
                let v    = store.len();
                let snap = store.snapshot_at(v);

                for (id, item) in &snap {
                    if item.text.is_empty() {
                        errors.lock().unwrap()
                              .push(format!("r{rid}: id={id} empty text"));
                    }
                }
                if rid == 0 {
                    samples.lock().unwrap().push((rid, v, snap.len()));
                }
                thread::sleep(Duration::from_millis(5));
            }
        }));
    }

    // ── Join & report ─────────────────────────────────────────────────
    writer.join().expect("writer panicked");
    for h in handles { h.join().expect("reader panicked"); }

    {
        let errs = errors.lock().unwrap();
        if errs.is_empty() {
            println!("  \u{2713} 5 readers x 30 iters = 150 ops — zero errors.");
        } else {
            eprintln!("  \u{2717} {} error(s): {:?}", errs.len(), *errs);
            std::process::exit(1);
        }
    }

    println!("\n  Sample reads by reader-0 (reader, version, items):");
    for t in samples.lock().unwrap().iter().take(8) {
        println!("    {:?}", t);
    }

    let snap = latest.read().unwrap().current_snapshot();
    println!("\n  Final snapshot ({} items):", snap.len());
    for (id, item) in &snap {
        println!("    id={id}  {:?}", item);
    }
}

// ═════════════════════════════════════════════════════════════════════════

fn main() {
    part_a();
    part_b();
    println!("\n  Done.");
}

// ═════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_add(id: u64, text: &str) -> Event {
        Event::AddItem { id, text: text.to_string() }
    }
    fn mk_complete(id: u64) -> Event { Event::CompleteItem { id } }
    fn mk_rename(id: u64, t: &str) -> Event {
        Event::RenameItem { id, new_text: t.to_string() }
    }
    fn mk_delete(id: u64) -> Event { Event::DeleteItem { id } }

    // ── 1. append_event immutability ──────────────────────────────────

    #[test]
    fn old_store_unchanged_after_append() {
        let s0 = Store::new();
        let s1 = s0.append_event(mk_add(1, "Buy milk"));
        let _s2 = s1.append_event(mk_complete(1));
        assert_eq!(s0.len(), 0);
        assert_eq!(s1.len(), 1);
        let snap = s1.snapshot_at(1);
        assert_eq!(snap[&1].text, "Buy milk");
        assert!(!snap[&1].completed);
    }

    // ── 2. snapshot_at correctness ────────────────────────────────────

    #[test]
    fn snapshot_at_v0_is_empty() {
        let store = Store::new().append_event(mk_add(1, "A"));
        assert!(store.snapshot_at(0).is_empty());
    }

    #[test]
    fn snapshot_all_event_types() {
        let store = Store::new()
            .append_event(mk_add(1, "Buy milk"))
            .append_event(mk_add(2, "Write code"))
            .append_event(mk_complete(1))
            .append_event(mk_rename(2, "Write tests"))
            .append_event(mk_delete(1));

        assert!(store.snapshot_at(0).is_empty());

        let s1 = store.snapshot_at(1);
        assert_eq!(s1[&1].text, "Buy milk");
        assert!(!s1[&1].completed);

        assert!(store.snapshot_at(3)[&1].completed);
        assert_eq!(store.snapshot_at(4)[&2].text, "Write tests");

        let s5 = store.snapshot_at(5);
        assert!(!s5.contains_key(&1));
        assert!(s5.contains_key(&2));
    }

    // ── 3. Time travel / non-interference ────────────────────────────

    #[test]
    fn time_travel_non_interference() {
        let s1 = Store::new().append_event(mk_add(1, "Task"));
        let before = s1.snapshot_at(1);
        let s2 = s1.append_event(mk_complete(1));
        let after = s1.snapshot_at(1);

        assert_eq!(before, after);
        assert!(!before[&1].completed);
        assert!(s2.snapshot_at(2)[&1].completed);

        let s0 = Store::new();
        let s3 = s0.append_event(mk_add(1, "A"))
                   .append_event(mk_add(2, "B"))
                   .append_event(mk_complete(1));

        assert!(s0.current_snapshot().is_empty());
        assert!(!s3.snapshot_at(1)[&1].completed);
        assert!(s3.snapshot_at(3)[&1].completed);
    }

    // ── 4. history ────────────────────────────────────────────────────

    #[test]
    fn history_length() {
        let store = (1..=5).fold(Store::new(), |s, i| s.append_event(mk_add(i, "x")));
        assert_eq!(store.history().len(), 6);
    }

    #[test]
    fn history_every_n_versions() {
        let store = (1..=10).fold(Store::new(), |s, i| s.append_event(mk_add(i, "x")));
        let versions: Vec<usize> = store.history_every_n(3).iter().map(|(v, _)| *v).collect();
        assert_eq!(versions, vec![0, 3, 6, 9]);
    }

    // ── 5. CheckpointStore matches plain Store ────────────────────────

    #[test]
    fn checkpoint_store_agrees_with_plain() {
        let events = vec![
            mk_add(1,"A"), mk_add(2,"B"), mk_complete(1),
            mk_rename(2,"B2"), mk_delete(1),
        ];
        let plain = events.iter().cloned()
            .fold(Store::new(), |s, e| s.append_event(e));
        let cp = events.into_iter()
            .fold(CheckpointStore::new(2), |s, e| s.append_event(e));
        for v in 0..=plain.len() {
            assert_eq!(plain.snapshot_at(v), cp.snapshot_at(v),
                       "mismatch at v={v}");
        }
    }

    // ── 6. CachingStore returns same result twice ─────────────────────

    #[test]
    fn caching_store_idempotent() {
        let cs = CachingStore::new(
            Store::new()
                .append_event(mk_add(1, "A"))
                .append_event(mk_complete(1))
        );
        assert_eq!(cs.snapshot_at(2), cs.snapshot_at(2));
        assert!(cs.snapshot_at(2)[&1].completed);
    }

    // ── 7. Concurrency sanity ─────────────────────────────────────────

    #[test]
    fn concurrent_readers_never_see_torn_state() {
        let latest: Arc<RwLock<Arc<Store>>> =
            Arc::new(RwLock::new(Arc::new(Store::new())));
        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let writer = {
            let latest = Arc::clone(&latest);
            thread::spawn(move || {
                for i in 0..20u64 {
                    let new_store: Arc<Store> = {
                        let guard = latest.read().unwrap();
                        Arc::new(guard.append_event(Event::AddItem {
                            id: i + 1,
                            text: format!("item-{i}"),
                        }))
                    };
                    *latest.write().unwrap() = new_store;
                    thread::sleep(Duration::from_millis(5));
                }
            })
        };

        let mut handles = vec![];
        for rid in 0..5_usize {
            let latest = Arc::clone(&latest);
            let errors = Arc::clone(&errors);
            handles.push(thread::spawn(move || {
                for _ in 0..50 {
                    let store: Arc<Store> = Arc::clone(&*latest.read().unwrap());
                    let v = store.len();
                    let snap = store.snapshot_at(v);
                    for (id, item) in &snap {
                        if item.text.is_empty() {
                            errors.lock().unwrap()
                                  .push(format!("r{rid} id={id} empty"));
                        }
                    }
                    thread::sleep(Duration::from_millis(2));
                }
            }));
        }

        writer.join().unwrap();
        for h in handles { h.join().unwrap(); }
        assert!(errors.lock().unwrap().is_empty(),
                "errors: {:?}", errors.lock().unwrap());
    }
}
