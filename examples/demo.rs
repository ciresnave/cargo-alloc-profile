//! Example: Using allocation tracking to optimize any Rust code
//!
//! This demonstrates how to use the TrackingAllocator pattern to find
//! and eliminate unnecessary allocations in performance-critical code.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Global allocation tracking
static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static TOTAL_ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_MEMORY_USAGE: AtomicUsize = AtomicUsize::new(0);

struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: System is the standard allocator
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
            let new_total =
                TOTAL_ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();

            // Track peak memory usage
            let mut peak = PEAK_MEMORY_USAGE.load(Ordering::Relaxed);
            while new_total > peak {
                match PEAK_MEMORY_USAGE.compare_exchange_weak(
                    peak,
                    new_total,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        TOTAL_ALLOCATED_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        // SAFETY: System is the standard allocator, ptr/layout come from alloc
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

/// Allocation statistics snapshot
#[derive(Debug, Clone)]
struct AllocStats {
    allocations: usize,
    deallocations: usize,
    current_bytes: usize,
    peak_bytes: usize,
}

impl AllocStats {
    fn snapshot() -> Self {
        AllocStats {
            allocations: ALLOCATION_COUNT.load(Ordering::Relaxed),
            deallocations: DEALLOCATION_COUNT.load(Ordering::Relaxed),
            current_bytes: TOTAL_ALLOCATED_BYTES.load(Ordering::Relaxed),
            peak_bytes: PEAK_MEMORY_USAGE.load(Ordering::Relaxed),
        }
    }

    fn delta(&self, before: &AllocStats) -> Self {
        AllocStats {
            allocations: self.allocations - before.allocations,
            deallocations: self.deallocations - before.deallocations,
            current_bytes: self.current_bytes.saturating_sub(before.current_bytes),
            peak_bytes: self.peak_bytes.max(before.peak_bytes),
        }
    }
}

/// Macro to profile allocations in a code block
macro_rules! profile_allocations {
    ($label:expr, $code:block) => {{
        let before = AllocStats::snapshot();
        let result = $code;
        let after = AllocStats::snapshot();
        let delta = after.delta(&before);
        println!("\n📊 Allocation Profile: {}", $label);
        println!("   Allocations: {}", delta.allocations);
        println!("   Deallocations: {}", delta.deallocations);
        println!(
            "   Net allocations: {}",
            delta.allocations.saturating_sub(delta.deallocations)
        );
        println!(
            "   Bytes allocated: {} ({:.2} KB)",
            delta.current_bytes,
            delta.current_bytes as f64 / 1024.0
        );
        println!(
            "   Peak memory: {} ({:.2} KB)",
            delta.peak_bytes,
            delta.peak_bytes as f64 / 1024.0
        );
        result
    }};
}

fn main() {
    println!("🔍 Allocation Profiling Examples\n");

    // Example 1: String concatenation (inefficient)
    profile_allocations!("String concatenation (BAD)", {
        let mut s = String::new();
        for i in 0..1000 {
            s = s + &i.to_string(); // ❌ Reallocates on every iteration!
        }
        s
    });

    // Example 2: String concatenation (efficient)
    profile_allocations!("String concatenation (GOOD)", {
        let mut s = String::with_capacity(4000); // Pre-allocate
        for i in 0..1000 {
            s.push_str(&i.to_string()); // ✅ Reuses buffer
        }
        s
    });

    // Example 3: Vec growth (inefficient)
    profile_allocations!("Vec push without capacity (BAD)", {
        let mut v = Vec::new();
        for i in 0..1000 {
            v.push(i); // Reallocates multiple times
        }
        v
    });

    // Example 4: Vec growth (efficient)
    profile_allocations!("Vec push with capacity (GOOD)", {
        let mut v = Vec::with_capacity(1000); // Single allocation
        for i in 0..1000 {
            v.push(i);
        }
        v
    });

    // Example 5: HashMap (inefficient)
    profile_allocations!("HashMap without capacity (BAD)", {
        let mut map = std::collections::HashMap::new();
        for i in 0..1000 {
            map.insert(i, i * 2);
        }
        map
    });

    // Example 6: HashMap (efficient)
    profile_allocations!("HashMap with capacity (GOOD)", {
        let mut map = std::collections::HashMap::with_capacity(1000);
        for i in 0..1000 {
            map.insert(i, i * 2);
        }
        map
    });

    // Example 7: Zero-copy string slicing
    profile_allocations!("String slicing (zero-copy)", {
        let s = "Hello, World! This is a test string.";
        let slice1 = &s[0..5]; // No allocation
        let slice2 = &s[7..12]; // No allocation
        let slice3 = &s[14..]; // No allocation
        (slice1, slice2, slice3)
    });

    // Example 8: Clone vs reference
    let large_data = vec![0u8; 1_000_000];

    profile_allocations!("Cloning large data (BAD)", {
        let _copy = large_data.clone(); // 1MB allocation!
    });

    profile_allocations!("Borrowing large data (GOOD)", {
        let _reference = &large_data; // Zero allocations!
    });

    println!("\n✅ Profiling complete!");
    println!("\n💡 Key Takeaways:");
    println!("   1. Pre-allocate collections with with_capacity()");
    println!("   2. Use references instead of clones when possible");
    println!("   3. Prefer push_str/extend over + for string concatenation");
    println!("   4. Use slices for zero-copy views into data");
}
