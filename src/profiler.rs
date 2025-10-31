use backtrace::Backtrace;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Global flag to enable/disable profiling - starts disabled
static PROFILING_ACTIVE: AtomicBool = AtomicBool::new(false);

// Thread-local reentrancy guard - prevents infinite recursion
thread_local! {
    static IN_PROFILER: Cell<bool> = Cell::new(false);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSite {
    pub count: usize,
    pub total_bytes: usize,
    pub frames: Vec<String>,
}

pub struct ProfilerData {
    pub total_allocations: AtomicUsize,
    pub total_deallocations: AtomicUsize,
    pub total_bytes_allocated: AtomicUsize,
    pub peak_memory: AtomicUsize,
    pub current_memory: AtomicUsize,
    pub allocation_sites: Mutex<HashMap<String, AllocationSite>>,
}

static PROFILER: Lazy<ProfilerData> = Lazy::new(|| ProfilerData {
    total_allocations: AtomicUsize::new(0),
    total_deallocations: AtomicUsize::new(0),
    total_bytes_allocated: AtomicUsize::new(0),
    peak_memory: AtomicUsize::new(0),
    current_memory: AtomicUsize::new(0),
    allocation_sites: Mutex::new(HashMap::new()),
});

pub struct AllocationProfiler;
impl AllocationProfiler {
    pub fn record_allocation(size: usize, mut backtrace: Backtrace) {
        // Quick atomic check (no allocation)
        if !PROFILING_ACTIVE.load(Ordering::Relaxed) {
            return;
        }

        // Check for reentrancy - prevent infinite recursion
        let already_in_profiler = IN_PROFILER.with(|flag| {
            if flag.get() {
                true
            } else {
                flag.set(true);
                false
            }
        });

        if already_in_profiler {
            return;
        }

        // Update global counters
        PROFILER.total_allocations.fetch_add(1, Ordering::Relaxed);
        PROFILER
            .total_bytes_allocated
            .fetch_add(size, Ordering::Relaxed);

        let new_current = PROFILER.current_memory.fetch_add(size, Ordering::Relaxed) + size;

        // Update peak memory
        let mut peak = PROFILER.peak_memory.load(Ordering::Relaxed);
        while new_current > peak {
            match PROFILER.peak_memory.compare_exchange_weak(
                peak,
                new_current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }

        // Resolve backtrace and record allocation site
        backtrace.resolve();
        let frames = extract_frames(&backtrace);

        if !frames.is_empty() {
            let key = frames.join("\n");
            let mut sites = PROFILER.allocation_sites.lock();

            sites
                .entry(key)
                .and_modify(|site| {
                    site.count += 1;
                    site.total_bytes += size;
                })
                .or_insert_with(|| AllocationSite {
                    count: 1,
                    total_bytes: size,
                    frames,
                });
        }

        // Clear the reentrancy flag
        IN_PROFILER.with(|flag| flag.set(false));
    }

    pub fn record_deallocation(size: usize) {
        // Only record if profiling is active
        if !PROFILING_ACTIVE.load(Ordering::Relaxed) {
            return;
        }

        PROFILER.total_deallocations.fetch_add(1, Ordering::Relaxed);
        PROFILER.current_memory.fetch_sub(size, Ordering::Relaxed);
    }

    pub fn get_snapshot() -> ProfileSnapshot {
        let sites = PROFILER.allocation_sites.lock();

        ProfileSnapshot {
            total_allocations: PROFILER.total_allocations.load(Ordering::Relaxed),
            total_deallocations: PROFILER.total_deallocations.load(Ordering::Relaxed),
            total_bytes_allocated: PROFILER.total_bytes_allocated.load(Ordering::Relaxed),
            peak_memory: PROFILER.peak_memory.load(Ordering::Relaxed),
            current_memory: PROFILER.current_memory.load(Ordering::Relaxed),
            allocation_sites: sites.clone(),
        }
    }

    /// Enable allocation profiling
    pub fn enable() {
        PROFILING_ACTIVE.store(true, Ordering::Relaxed);
    }

    /// Disable allocation profiling
    pub fn disable() {
        PROFILING_ACTIVE.store(false, Ordering::Relaxed);
    }

    /// Write the profiling report to the configured output file
    pub fn write_report() {
        if let Ok(output_path) = std::env::var("CARGO_ALLOC_PROFILE_OUTPUT") {
            // Disable profiling during report generation
            Self::disable();

            let snapshot = Self::get_snapshot();
            if let Ok(json) = serde_json::to_string(&snapshot) {
                let _ = std::fs::write(&output_path, json);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub total_bytes_allocated: usize,
    pub peak_memory: usize,
    pub current_memory: usize,
    pub allocation_sites: HashMap<String, AllocationSite>,
}

fn extract_frames(backtrace: &Backtrace) -> Vec<String> {
    let mut frames = Vec::new();
    let mut skip_frames = 0;

    for frame in backtrace.frames() {
        for symbol in frame.symbols() {
            if let Some(name) = symbol.name() {
                let name_str = name.to_string();

                // Skip internal allocator and profiler frames
                if name_str.contains("alloc::")
                    || name_str.contains("ProfilingAllocator")
                    || name_str.contains("AllocationProfiler")
                    || name_str.contains("backtrace::")
                {
                    skip_frames += 1;
                    continue;
                }

                // Take meaningful frames (limit to prevent huge stacks)
                if skip_frames > 0 && frames.len() < 10 {
                    // Clean up the symbol name
                    let clean_name = clean_symbol_name(&name_str);

                    // Include file and line if available
                    let location =
                        if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                            format!("{} ({}:{})", clean_name, file.display(), line)
                        } else {
                            clean_name
                        };

                    frames.push(location);
                }
            }
        }
    }

    frames
}

fn clean_symbol_name(name: &str) -> String {
    // Remove hash suffixes like ::h1a2b3c4d5e6f7g8
    let name = if let Some(pos) = name.rfind("::h") {
        if name[pos + 3..].chars().all(|c| c.is_ascii_hexdigit()) {
            &name[..pos]
        } else {
            name
        }
    } else {
        name
    };

    // Simplify generic parameters
    let name = name.replace("<", "‹").replace(">", "›");

    name.to_string()
}
