#![allow(unsafe_code)]

use crate::xor_list::XorList;

mod xor_list;
mod test;

fn main() {
    println!("══════════════════════════════════════════");
    println!(" XOR Linked List — demonstration");
    println!("══════════════════════════════════════════\n");

    let mut list: XorList<i32> = xor_list::new();

    println!("push_back 10, 20, 30, 40, 50");
    for v in [10, 20, 30, 40, 50] {
        list.push_back(v);
    }
    print!("Forward  : ");
    list.traverse(|v| print!("{v} "));
    println!();
    print!("Reverse  : ");
    list.traverse_reverse(|v| print!("{v} "));
    println!("\n");

    println!("push_front 0");
    list.push_front(0);
    print!("Forward  : ");
    list.traverse(|v| print!("{v} "));
    println!("\n");

    println!("insert 25 at index 3  (between 20 and 30)");
    list.insert(3, 25);
    print!("Forward  : ");
    list.traverse(|v| print!("{v} "));
    println!("\n");

    println!("delete at index 0 (head)");
    let removed = list.delete(0);
    println!("Removed  : {:?}", removed);
    print!("Forward  : ");
    list.traverse(|v| print!("{v} "));
    println!();

    println!("\ndelete at last index (tail)");
    let removed = list.delete(list.len() - 1);
    println!("Removed  : {:?}", removed);
    print!("Forward  : ");
    list.traverse(|v| print!("{v} "));
    println!();

    println!("\ndelete index 2 (middle element)");
    let removed = list.delete(2);
    println!("Removed  : {:?}", removed);
    print!("Forward  : ");
    list.traverse(|v| print!("{v} "));
    println!();

    println!("\nSum of all elements via traverse closure:");
    let mut sum = 0i32;
    list.traverse(|v| sum += v);
    println!("Sum = {sum}");

    println!("\nDebug display: {:?}", list);

    println!("\n── String list (tests ownership / drop) ──");
    let mut slist: XorList<i32> = xor_list::new();
    for word in ["alpha", "beta", "gamma", "delta"] {
        slist.push_back(word.to_string().parse().unwrap());
    }
    slist.insert(2, "INSERTED".to_string().parse().unwrap());
    print!("Forward  : ");
    slist.traverse(|s| print!("{s} "));
    println!();
    let got = slist.delete(2).unwrap();
    println!("Deleted  : {got}");
    print!("Forward  : ");
    slist.traverse(|s| print!("{s} "));
    println!();

    println!("\n✓  All demonstrations complete — list drops cleanly here.");
}