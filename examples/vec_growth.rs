//! Example showing Vec growth patterns and optimization
//!
//! Run with: cargo alloc-profile run --example vec_growth

use cargo_alloc_profile::AllocationProfiler;

fn grow_without_capacity(size: usize) -> Vec<i32> {
    let mut vec = Vec::new(); // ❌ Will reallocate multiple times
    for i in 0..size {
        vec.push(i as i32);
    }
    vec
}

fn grow_with_capacity(size: usize) -> Vec<i32> {
    let mut vec = Vec::with_capacity(size); // ✅ Single allocation
    for i in 0..size {
        vec.push(i as i32);
    }
    vec
}

fn collect_iterator(size: usize) -> Vec<i32> {
    (0..size as i32).collect() // ✅ Efficient
}

fn main() {
    AllocationProfiler::enable();

    println!("Vec Growth Example\n");

    let size = 100;

    println!("Method 1: Growing without capacity (multiple reallocations)");
    let _v1 = grow_without_capacity(size);

    println!("Method 2: Growing with capacity (single allocation)");
    let _v2 = grow_with_capacity(size);

    println!("Method 3: Using collect (efficient)");
    let _v3 = collect_iterator(size);

    println!("\nCompare allocation counts:");
    println!("Without capacity: ~7-8 reallocations");
    println!("With capacity: 1 allocation");
    println!("Using collect: 1 allocation\n");

    AllocationProfiler::write_report();
}
