/// Partition `data` in-place using the last element as pivot.
/// Returns the final index of the pivot.
fn partition<T: Ord>(data: &mut [T]) -> usize {
    let len = data.len();
    let mid = len / 2;
    let last = len - 1;

    if data[0] > data[mid] {
        data.swap(0, mid);
    }
    if data[0] > data[last] {
        data.swap(0, last);
    }
    if data[mid] > data[last] {
        data.swap(mid, last);
    }
    data.swap(mid, last - 1);
    let pivot_idx = last - 1;

    if len <= 3 {
        return mid;
    }

    let mut i = 0usize;
    data.swap(pivot_idx, last);
    let pivot_pos = last;

    let mut j = 0usize;
    while j < pivot_pos {
        if data[j] <= data[pivot_pos] {
            data.swap(i, j);
            i += 1;
        }
        j += 1;
    }
    data.swap(i, pivot_pos);
    i
}

/// Recursive in-place Quicksort (sequential).
///
/// # Examples
/// ```
/// use assignment07::sequential_quicksort;
/// let mut v = vec![5, 3, 8, 1, 9, 2];
/// sequential_quicksort(&mut v);
/// assert_eq!(v, vec![1, 2, 3, 5, 8, 9]);
/// ```
pub fn sequential_quicksort<T: Ord>(data: &mut [T]) {
    if data.len() <= 1 {
        return;
    }
    if data.len() == 2 {
        if data[0] > data[1] {
            data.swap(0, 1);
        }
        return;
    }

    let pivot = partition(data);

    sequential_quicksort(&mut data[..pivot]);
    sequential_quicksort(&mut data[pivot + 1..]);
}

// ------------------------------------------------------------
// PART 2 — Parallel Quicksort
// ------------------------------------------------------------

/// Parallel in-place Quicksort using `std::thread::scope`.
///
/// * Partitions the slice around a pivot.
/// * When both halves are larger than `cutoff`, spawns one thread for the
///   left half and recurses on the right half on the current thread.
/// * Falls back to `sequential_quicksort` for slices ≤ `cutoff`.
///
/// # Safety / Ownership
/// `split_at_mut` produces two *disjoint* mutable references, which Rust
/// verifies at compile time.  `thread::scope` ensures all spawned threads
/// finish before the scope exits, so the borrows remain valid.
///
/// # Examples
/// ```
/// use assignment07::parallel_quicksort;
/// let mut v = vec![5, 3, 8, 1, 9, 2];
/// parallel_quicksort(&mut v, 1024);
/// assert_eq!(v, vec![1, 2, 3, 5, 8, 9]);
/// ```
pub fn parallel_quicksort<T: Ord + Send>(data: &mut [T], cutoff: usize) {
    if data.len() <= cutoff {
        sequential_quicksort(data);
        return;
    }

    let pivot = partition(data);
    let (left, right_with_pivot) = data.split_at_mut(pivot);
    let right = &mut right_with_pivot[1..];

    if left.len() > cutoff && right.len() > cutoff {
        std::thread::scope(|s| {
            s.spawn(|| {
                parallel_quicksort(left, cutoff);
            });
            parallel_quicksort(right, cutoff);
        });
    } else {
        parallel_quicksort(left, cutoff);
        parallel_quicksort(right, cutoff);
    }
}