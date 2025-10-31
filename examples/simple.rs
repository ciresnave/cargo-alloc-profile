//! Simple test program to demonstrate cargo-alloc-profile
//!
//! Run with: cargo alloc-profile run --example simple

use cargo_alloc_profile::AllocationProfiler;

fn main() {
    // Enable profiling unconditionally (the cargo wrapper sets the env var before this runs)
    AllocationProfiler::enable();

    println!("Running allocation test...\n");

    // Test 1: Vec without capacity - will reallocate
    let mut items = Vec::new();
    for i in 0..100 {
        items.push(i);
    }
    println!("Created {} items without capacity", items.len());

    // Test 2: Vec with capacity - single allocation
    let mut items_optimized = Vec::with_capacity(100);
    for i in 0..100 {
        items_optimized.push(i);
    }
    println!("Created {} items with capacity", items_optimized.len());

    // Test 3: String allocations
    let mut strings = Vec::new();
    for i in 0..50 {
        strings.push(format!("Item {}", i));
    }
    println!("Created {} strings", strings.len());

    println!("\nDone!");

    // Write profiling report before exit
    AllocationProfiler::write_report();
}
