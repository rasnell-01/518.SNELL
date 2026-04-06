# Assignment 7 — Parallel Quicksort in Rust

## Overview

This project implements **sequential** and **parallel** in-place Quicksort in
Rust using only the standard library (`std::thread`, `std::thread::scope`).

## Project Layout

```
Assignment07/
├── Cargo.toml
├── README.md
├── report.pdf
└── src/
    ├── lib.rs          # sequential_quicksort + parallel_quicksort
    └── benchmark.rs    # benchmark binary
```

## Building

```bash
# debug build (includes tests)
cargo build

# optimised release build (required for benchmarks)
cargo build --release
```

## Running Tests

```bash
cargo test
```

All 18 unit tests cover:

| Category | Cases |
|----------|-------|
| Sequential | empty, single, two elements, random, sorted, reverse, all-duplicates, duplicate-heavy, large random |
| Parallel | empty, single, random, sorted, reverse, all-duplicates, duplicate-heavy, large random, matches sequential |

## Running Benchmarks

```bash
cargo run --release --bin benchmark
```

The benchmark driver produces three tables:

| Table | Description |
|-------|-------------|
| 1 | Sequential vs Parallel (cutoff = 4 096) across all sizes and input types |
| 2 | Cutoff analysis — 256 / 1 024 / 4 096 / 16 384 for random 1 M elements |
| 3 | Full speedup matrix: all cutoffs × all sizes × all input types |

## API

```rust
/// In-place sequential Quicksort — works for any Ord type.
pub fn sequential_quicksort<T: Ord>(data: &mut [T]);

/// In-place parallel Quicksort.
/// Falls back to sequential when slice length ≤ cutoff.
pub fn parallel_quicksort<T: Ord + Send>(data: &mut [T], cutoff: usize);
```

## Design Highlights

### Pivot Strategy — Median of Three

```
data[0]  data[mid]  data[last]
```

The three elements are sorted in-place so the median lands at `data[mid]`,
then swapped to `data[last-1]` before Lomuto partitioning.  This eliminates
the O(n²) worst case on already-sorted input.

### Disjoint Mutable Slices

```rust
let (left, right_with_pivot) = data.split_at_mut(pivot);
let right = &mut right_with_pivot[1..];
```

`split_at_mut` produces two non-overlapping `&mut [T]` references that Rust
verifies at compile time — no unsafe code is needed.

### Scoped Threads

```rust
std::thread::scope(|s| {
    s.spawn(|| parallel_quicksort(left,  cutoff));
    parallel_quicksort(right, cutoff);
});
```

`thread::scope` guarantees all spawned threads complete before the scope
exits, so the borrowed slices remain valid.  No `Arc`, `Mutex`, or `unsafe`
is required.

### Cutoff Threshold

When a slice is ≤ `cutoff` elements, the algorithm falls back to
`sequential_quicksort`.  A cutoff of **4 096** balances thread-creation
overhead against useful parallel work on most machines.

## Constraints Respected

- ✅ No `Rayon`, `Crossbeam`, `Tokio`, or other external crates
- ✅ No `slice::sort` used as the implementation
- ✅ No `Mutex`, `Arc`, or shared global state
- ✅ Only `std::thread::scope`, `split_at_mut`, and recursion
