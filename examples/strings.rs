//! Example showing string allocation patterns and optimizations
//!
//! Run with: cargo alloc-profile run --example strings

use cargo_alloc_profile::AllocationProfiler;

fn concatenate_unoptimized(strings: &[&str]) -> String {
    let mut result = String::new();
    for s in strings {
        result = result + s; // ❌ Creates new string each time
    }
    result
}

fn concatenate_pushstr(strings: &[&str]) -> String {
    let mut result = String::new();
    for s in strings {
        result.push_str(s); // ✅ Better, but still reallocates
    }
    result
}

fn concatenate_optimized(strings: &[&str]) -> String {
    let capacity: usize = strings.iter().map(|s| s.len()).sum();
    let mut result = String::with_capacity(capacity);
    for s in strings {
        result.push_str(s); // ✅ Single allocation
    }
    result
}

fn main() {
    AllocationProfiler::enable();

    println!("String Allocation Example\n");

    let words = vec![
        "Hello", " ", "world", "!", " ", "How", " ", "are", " ", "you", "?",
    ];

    println!("Method 1: Using + operator (many allocations)");
    let _result1 = concatenate_unoptimized(&words);

    println!("Method 2: Using push_str (fewer allocations)");
    let _result2 = concatenate_pushstr(&words);

    println!("Method 3: With capacity (optimal)");
    let _result3 = concatenate_optimized(&words);

    println!("\nCheck the allocation report below to see the difference!");

    AllocationProfiler::write_report();
}
