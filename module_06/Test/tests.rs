use safe_float::{SafeFloat, safe_div, safe_ln, safe_sqrt};
use command::{Command, insertion_sort_plan, insertion_sort, interpret};
mod command;
mod safe_float;

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Problem 1 tests ---------------------------------

    #[test]
    fn test_insertion_sort_basic() {
        let mut data = vec![5, 3, 8, 1, 2];
        insertion_sort(&mut data);
        assert_eq!(data, vec![1, 2, 3, 5, 8]);
    }

    #[test]
    fn test_insertion_sort_already_sorted() {
        let mut data = vec![1, 2, 3, 4, 5];
        insertion_sort(&mut data);
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_insertion_sort_reverse() {
        let mut data = vec![5, 4, 3, 2, 1];
        insertion_sort(&mut data);
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_insertion_sort_single() {
        let mut data = vec![42];
        insertion_sort(&mut data);
        assert_eq!(data, vec![42]);
    }

    #[test]
    fn test_insertion_sort_empty() {
        let mut data: Vec<i32> = vec![];
        insertion_sort(&mut data);
        assert_eq!(data, vec![]);
    }

    #[test]
    fn test_plan_is_pure() {
        // Calling the pure planner must not modify the original slice.
        let original = vec![3, 1, 2];
        let _cmds = insertion_sort_plan(&original);
        // original is unchanged because we only borrowed it
        assert_eq!(original, vec![3, 1, 2]);
    }

    // ---- Problem 2 tests ---------------------------------

    #[test]
    fn test_unit_valid() {
        assert_eq!(SafeFloat::of(3.14), SafeFloat::Value(3.14));
    }

    #[test]
    fn test_unit_nan() {
        assert_eq!(SafeFloat::of(f64::NAN), SafeFloat::NaN);
    }

    #[test]
    fn test_unit_infinite() {
        assert_eq!(SafeFloat::of(f64::INFINITY), SafeFloat::NaN);
    }

    #[test]
    fn test_map_valid() {
        let result = SafeFloat::of(4.0).map(|x| x * 2.0);
        assert_eq!(result, SafeFloat::Value(8.0));
    }

    #[test]
    fn test_map_nan_propagates() {
        let result = SafeFloat::NaN.map(|x| x * 2.0);
        assert_eq!(result, SafeFloat::NaN);
    }

    #[test]
    fn test_and_then_safe_sqrt() {
        let result = SafeFloat::of(16.0).and_then(safe_sqrt);
        assert_eq!(result, SafeFloat::Value(4.0));
    }

    #[test]
    fn test_and_then_safe_sqrt_negative() {
        let result = SafeFloat::of(-1.0).and_then(safe_sqrt);
        assert_eq!(result, SafeFloat::NaN);
    }

    #[test]
    fn test_and_then_div_by_zero() {
        // 10.0 / 0.0 → NaN
        let result = SafeFloat::of(0.0).and_then(safe_div(10.0));
        assert_eq!(result, SafeFloat::NaN);
    }

    #[test]
    fn test_chain_stays_nan_once_invalid() {
        // sqrt(-1) → NaN, then * 5, then ln — all NaN
        let result = SafeFloat::of(-1.0)
            .and_then(safe_sqrt)          // NaN
            .map(|x| x * 5.0)            // still NaN
            .and_then(safe_ln);           // still NaN
        assert_eq!(result, SafeFloat::NaN);
    }

    #[test]
    fn test_chain_valid() {
        // sqrt(100) = 10, ln(10) ≈ 2.302...
        let result = SafeFloat::of(100.0)
            .and_then(safe_sqrt)
            .and_then(safe_ln);
        match result {
            SafeFloat::Value(v) => assert!((v - 10f64.ln()).abs() < 1e-10),
            SafeFloat::NaN      => panic!("Expected a valid value"),
        }
    }

    #[test]
    fn test_unwrap_or() {
        assert_eq!(SafeFloat::Value(7.0).unwrap_or(0.0), 7.0);
        assert_eq!(SafeFloat::NaN.unwrap_or(-1.0), -1.0);
    }
}