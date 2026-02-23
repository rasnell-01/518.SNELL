use crate::xor_list::XorList;

#[cfg(test)]
mod tests {
    use super::*;

    fn to_vec<T: Clone>(list: &XorList<T>) -> Vec<T> {
        let mut v = Vec::new();
        list.traverse(|x| v.push(x.clone()));
        v
    }

    fn to_vec_rev<T: Clone>(list: &XorList<T>) -> Vec<T> {
        let mut v = Vec::new();
        list.traverse_reverse(|x| v.push(x.clone()));
        v
    }

    #[test]
    fn empty_list() {
        let list: XorList<i32> = XorList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(to_vec(&list), vec![] as Vec<i32>);
    }

    #[test]
    fn push_back_single() {
        let mut list = XorList::new();
        list.push_back(42);
        assert_eq!(list.len(), 1);
        assert_eq!(to_vec(&list), vec![42]);
    }

    #[test]
    fn push_front_single() {
        let mut list = XorList::new();
        list.push_front(99);
        assert_eq!(list.len(), 1);
        assert_eq!(to_vec(&list), vec![99]);
    }

    #[test]
    fn push_back_multiple() {
        let mut list = XorList::new();
        for i in 1..=5 {
            list.push_back(i);
        }
        assert_eq!(to_vec(&list), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn push_front_multiple() {
        let mut list = XorList::new();
        for i in 1..=5 {
            list.push_front(i);
        }
        assert_eq!(to_vec(&list), vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn traverse_forward_and_reverse_agree() {
        let mut list = XorList::new();
        for i in 1..=6 {
            list.push_back(i);
        }
        let fwd = to_vec(&list);
        let mut rev = to_vec_rev(&list);
        rev.reverse();
        assert_eq!(fwd, rev, "forward and reversed-reverse traversal must match");
    }

    #[test]
    fn insert_at_front() {
        let mut list = XorList::new();
        list.push_back(2);
        list.push_back(3);
        list.insert(0, 1);
        assert_eq!(to_vec(&list), vec![1, 2, 3]);
    }

    #[test]
    fn insert_at_back() {
        let mut list = XorList::new();
        list.push_back(1);
        list.push_back(2);
        list.insert(99, 3);
        assert_eq!(to_vec(&list), vec![1, 2, 3]);
    }

    #[test]
    fn insert_in_middle() {
        let mut list = XorList::new();
        for i in [1, 2, 4, 5] {
            list.push_back(i);
        }
        list.insert(2, 3);
        assert_eq!(to_vec(&list), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn insert_bidirectional_integrity() {
        let mut list = XorList::new();
        for i in [10, 30] {
            list.push_back(i);
        }
        list.insert(1, 20);
        let fwd = to_vec(&list);
        let mut rev = to_vec_rev(&list);
        rev.reverse();
        assert_eq!(fwd, vec![10, 20, 30]);
        assert_eq!(fwd, rev);
    }

    #[test]
    fn delete_only_element() {
        let mut list = XorList::new();
        list.push_back(7);
        assert_eq!(list.delete(0), Some(7));
        assert!(list.is_empty());
    }

    #[test]
    fn delete_head() {
        let mut list = XorList::new();
        for i in 1..=4 {
            list.push_back(i);
        }
        assert_eq!(list.delete(0), Some(1));
        assert_eq!(to_vec(&list), vec![2, 3, 4]);
    }

    #[test]
    fn delete_tail() {
        let mut list = XorList::new();
        for i in 1..=4 {
            list.push_back(i);
        }
        assert_eq!(list.delete(3), Some(4));
        assert_eq!(to_vec(&list), vec![1, 2, 3]);
    }

    #[test]
    fn delete_middle() {
        let mut list = XorList::new();
        for i in 1..=5 {
            list.push_back(i);
        }
        assert_eq!(list.delete(2), Some(3));
        assert_eq!(to_vec(&list), vec![1, 2, 4, 5]);
    }

    #[test]
    fn delete_out_of_bounds() {
        let mut list: XorList<i32> = XorList::new();
        assert_eq!(list.delete(0), None);
        list.push_back(1);
        assert_eq!(list.delete(1), None);
    }

    #[test]
    fn delete_maintains_reverse_integrity() {
        let mut list = XorList::new();
        for i in 1..=5 {
            list.push_back(i);
        }
        list.delete(2);
        let fwd = to_vec(&list);
        let mut rev = to_vec_rev(&list);
        rev.reverse();
        assert_eq!(fwd, rev, "reverse traversal must mirror forward after delete");
    }

    #[test]
    fn drop_with_owned_strings() {
        let mut list = XorList::new();
        list.push_back(String::from("hello"));
        list.push_back(String::from("world"));
        list.push_front(String::from("dear"));
        assert_eq!(to_vec(&list), vec!["dear", "hello", "world"]);
    }

    #[test]
    fn delete_returns_owned_value() {
        let mut list = XorList::new();
        list.push_back(String::from("alpha"));
        list.push_back(String::from("beta"));
        list.push_back(String::from("gamma"));
        let removed = list.delete(1).unwrap();
        assert_eq!(removed, "beta");
        assert_eq!(to_vec(&list), vec!["alpha", "gamma"]);
    }

    #[test]
    fn generic_vec_elements() {
        let mut list: XorList<Vec<i32>> = XorList::new();
        list.push_back(vec![1, 2]);
        list.push_back(vec![3, 4]);
        list.insert(1, vec![10, 20]);
        let result: Vec<Vec<i32>> = {
            let mut v = Vec::new();
            list.traverse(|x| v.push(x.clone()));
            v
        };
        assert_eq!(result, vec![vec![1, 2], vec![10, 20], vec![3, 4]]);
    }

    #[test]
    fn stress_push_and_delete() {
        const N: usize = 1_000;
        let mut list = XorList::new();

        for i in 0..N {
            list.push_back(i);
        }
        assert_eq!(list.len(), N);

        let mut expected: Vec<usize> = (0..N).collect();
        while !list.is_empty() {
            list.delete(0);
            expected.remove(0);
            assert_eq!(to_vec(&list), expected);
        }

        assert!(list.is_empty());
    }
}