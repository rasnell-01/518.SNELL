#[cfg(test)]
mod tests {
    use lib::{parallel_quicksort, sequential_quicksort};
    use super::*;

    fn is_sorted<T: Ord>(v: &[T]) -> bool {
        v.windows(2).all(|w| w[0] <= w[1])
    }

    // --- sequential ---

    #[test]
    fn seq_empty() {
        let mut v: Vec<i32> = vec![];
        sequential_quicksort(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn seq_single() {
        let mut v = vec![42];
        sequential_quicksort(&mut v);
        assert_eq!(v, vec![42]);
    }

    #[test]
    fn seq_two_elements() {
        let mut v = vec![9, 1];
        sequential_quicksort(&mut v);
        assert_eq!(v, vec![1, 9]);
    }

    #[test]
    fn seq_random() {
        let mut v = vec![5, 3, 8, 1, 9, 2, 7, 4, 6];
        sequential_quicksort(&mut v);
        assert!(is_sorted(&v));
    }

    #[test]
    fn seq_already_sorted() {
        let mut v: Vec<i32> = (0..1000).collect();
        sequential_quicksort(&mut v);
        assert!(is_sorted(&v));
    }

    #[test]
    fn seq_reverse_sorted() {
        let mut v: Vec<i32> = (0..1000).rev().collect();
        sequential_quicksort(&mut v);
        assert!(is_sorted(&v));
    }

    #[test]
    fn seq_all_duplicates() {
        let mut v = vec![7i32; 500];
        sequential_quicksort(&mut v);
        assert!(v.iter().all(|&x| x == 7));
    }

    #[test]
    fn seq_duplicate_heavy() {
        let mut v: Vec<i32> = (0..1000).map(|i| i % 10).collect();
        sequential_quicksort(&mut v);
        assert!(is_sorted(&v));
    }

    #[test]
    fn seq_large_random() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut v: Vec<i32> = (0..50_000)
            .map(|i| {
                let mut h = DefaultHasher::new();
                i.hash(&mut h);
                h.finish() as i32
            })
            .collect();
        sequential_quicksort(&mut v);
        assert!(is_sorted(&v));
    }

    // --- parallel ---

    #[test]
    fn par_empty() {
        let mut v: Vec<i32> = vec![];
        parallel_quicksort(&mut v, 1024);
        assert!(v.is_empty());
    }

    #[test]
    fn par_single() {
        let mut v = vec![42];
        parallel_quicksort(&mut v, 1024);
        assert_eq!(v, vec![42]);
    }

    #[test]
    fn par_random() {
        let mut v = vec![5, 3, 8, 1, 9, 2, 7, 4, 6];
        parallel_quicksort(&mut v, 4);
        assert!(is_sorted(&v));
    }

    #[test]
    fn par_already_sorted() {
        let mut v: Vec<i32> = (0..1000).collect();
        parallel_quicksort(&mut v, 256);
        assert!(is_sorted(&v));
    }

    #[test]
    fn par_reverse_sorted() {
        let mut v: Vec<i32> = (0..1000).rev().collect();
        parallel_quicksort(&mut v, 256);
        assert!(is_sorted(&v));
    }

    #[test]
    fn par_all_duplicates() {
        let mut v = vec![7i32; 500];
        parallel_quicksort(&mut v, 64);
        assert!(v.iter().all(|&x| x == 7));
    }

    #[test]
    fn par_duplicate_heavy() {
        let mut v: Vec<i32> = (0..1000).map(|i| i % 10).collect();
        parallel_quicksort(&mut v, 256);
        assert!(is_sorted(&v));
    }

    #[test]
    fn par_large_random() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut v: Vec<i32> = (0..100_000)
            .map(|i| {
                let mut h = DefaultHasher::new();
                i.hash(&mut h);
                h.finish() as i32
            })
            .collect();
        parallel_quicksort(&mut v, 4096);
        assert!(is_sorted(&v));
    }

    #[test]
    fn par_matches_seq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let base: Vec<i32> = (0..20_000)
            .map(|i| {
                let mut h = DefaultHasher::new();
                i.hash(&mut h);
                h.finish() as i32
            })
            .collect();
        let mut seq = base.clone();
        let mut par = base.clone();
        sequential_quicksort(&mut seq);
        parallel_quicksort(&mut par, 2048);
        assert_eq!(seq, par);
    }
}