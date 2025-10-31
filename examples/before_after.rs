//! Example showing before/after optimization
//!
//! Run twice to see the difference:
//! 1. cargo alloc-profile --save before.json run --example before_after -- before
//! 2. cargo alloc-profile --compare before.json run --example before_after -- after

use cargo_alloc_profile::AllocationProfiler;
use std::env;

#[derive(Clone)]
struct Data {
    values: Vec<i32>,
    label: String,
}

// Version 1: Unoptimized (makes many clones)
fn process_unoptimized(data: &Data, count: usize) -> Vec<Data> {
    let mut results = Vec::new();
    for i in 0..count {
        let mut item = data.clone(); // ❌ Expensive clone
        item.label = format!("{}-{}", data.label, i); // ❌ New allocation each time
        results.push(item);
    }
    results
}

// Version 2: Optimized (pre-allocate and minimize clones)
fn process_optimized(data: &Data, count: usize) -> Vec<Data> {
    let mut results = Vec::with_capacity(count); // ✅ Pre-allocate
    for i in 0..count {
        let label_capacity = data.label.len() + 10; // Estimate size
        let mut label = String::with_capacity(label_capacity);
        label.push_str(&data.label);
        label.push('-');
        label.push_str(&i.to_string());

        results.push(Data {
            values: data.values.clone(), // Still need clone, but at least we pre-allocate
            label,
        });
    }
    results
}

fn main() {
    AllocationProfiler::enable();

    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("before");

    let data = Data {
        values: vec![1, 2, 3, 4, 5],
        label: "item".to_string(),
    };

    println!("Running {} optimization\n", mode);

    let _results = match mode {
        "after" => {
            println!("✅ Using optimized version with pre-allocation");
            process_optimized(&data, 50)
        }
        _ => {
            println!("❌ Using unoptimized version");
            process_unoptimized(&data, 50)
        }
    };

    println!("\nTo compare:");
    println!("1. cargo alloc-profile --save before.json run --example before_after -- before");
    println!("2. cargo alloc-profile --compare before.json run --example before_after -- after\n");

    AllocationProfiler::write_report();
}
