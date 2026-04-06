use safe_float::{SafeFloat, safe_div, safe_ln, safe_sqrt};
use command::{Command, insertion_sort_plan, insertion_sort, interpret};
mod command;
mod safe_float;

fn main() {
    // --- Problem 1 demo ---
    println!("=== Problem 1: Insertion Sort (Pure / Impure separation) ===");
    let mut data = vec![64, 25, 12, 22, 11];
    println!("Before: {:?}", data);

    let plan = insertion_sort_plan(&data);
    println!("Swap plan ({} commands): {:?}", plan.len(), plan);

    interpret(&plan, &mut data);
    println!("After : {:?}\n", data);

    // --- Problem 2 demo ---
    println!("=== Problem 2: SafeFloat Monad ===");

    // Happy path: sqrt(256) / 4 = 4
    let happy = SafeFloat::of(256.0)
        .and_then(safe_sqrt)          // 16.0
        .and_then(safe_div(64.0))     // 64 / 16 = 4.0
        .map(|x| x - 10.0);          // 4 - 10 = -6.0
    println!("Happy path  sqrt(256) → /4 → -10  = {:?}", happy);

    // NaN injection: sqrt of a negative number poisons the chain
    let poisoned = SafeFloat::of(-9.0)
        .and_then(safe_sqrt)          // NaN
        .map(|x| x + 100.0)          // still NaN
        .and_then(safe_ln);           // still NaN
    println!("Poisoned    sqrt(-9) → +100 → ln  = {:?}", poisoned);

    // Divide-by-zero path
    let div_zero = SafeFloat::of(0.0)
        .and_then(safe_div(42.0))     // NaN
        .map(|x| x * 3.0);           // still NaN
    println!("Div-by-zero 42/0 → *3              = {:?}", div_zero);

    // unwrap_or provides a safe fallback
    println!("Fallback value from NaN: {}", poisoned.unwrap_or(f64::NAN));
}