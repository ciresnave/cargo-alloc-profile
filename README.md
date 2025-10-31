# cargo-alloc-profile

[![Crates.io](https://img.shields.io/crates/v/cargo-alloc-profile.svg)](https://crates.io/crates/cargo-alloc-profile)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/ciresnave/cargo-alloc-profile)

A powerful cargo subcommand for profiling heap allocations in Rust programs. Identify allocation hotspots, track memory usage, and optimize your code with detailed allocation reports.

## Features

- 🔍 **Zero-overhead profiling** - Only profiles when enabled, minimal runtime impact
- 📊 **Detailed reports** - Function-level allocation tracking with call stacks
- 🎯 **Flexible filtering** - Filter by function name, minimum count, or byte threshold
- 📈 **Multiple output formats** - Human-readable text or JSON for tool integration
- 🔄 **Baseline comparison** - Compare runs to track allocation changes over time
- 📦 **Grouping options** - Aggregate by function, module, or file
- ⚡ **Fast** - Uses efficient atomic operations and lock-free data structures
## Installation

```bash
cargo install cargo-alloc-profile
```

## Quick Start

Profile your application:

```bash
cargo alloc-profile run --example simple
```

Profile with filtering and sorting:

```bash
cargo alloc-profile --min-count 10 --sort-by size -v run --bin myapp
```

Save a baseline and compare:

```bash
# Save baseline
cargo alloc-profile --save baseline.json run

# Make changes, then compare
cargo alloc-profile --compare baseline.json run
```

## Usage

### Basic Commands

```bash
# Profile a binary
cargo alloc-profile run --bin myapp

# Profile an example
cargo alloc-profile run --example demo

# Profile tests
cargo alloc-profile test

# Profile benchmarks
cargo alloc-profile bench
```

### Filtering Options

```bash
# Only show allocations that occurred at least 5 times
cargo alloc-profile --min-count 5 run

# Only show allocations over 1KB
cargo alloc-profile --threshold-bytes 1024 run

# Filter by function name
cargo alloc-profile -f "string" run

# Show only top 10 allocation sites
cargo alloc-profile --limit 10 run
```

### Sorting and Display

```bash
# Sort by total bytes allocated
cargo alloc-profile --sort-by size run

# Sort alphabetically by function name
cargo alloc-profile --sort-by name run

# Increase verbosity for more details
cargo alloc-profile -v run        # Show bytes
cargo alloc-profile -vv run       # Show file locations
cargo alloc-profile -vvv run      # Show stack traces
```

### Grouping

```bash
# Group by module (default: function)
cargo alloc-profile --group-by module run

# Group by source file
cargo alloc-profile --group-by file run
```

### Output Formats

```bash
# JSON output (great for LLMs and tooling)
cargo alloc-profile -o json run

# JSON with full details
cargo alloc-profile -o json -vvv run
```

### Comparison Mode

```bash
# Save current run as baseline
cargo alloc-profile --save baseline.json run

# Compare against baseline
cargo alloc-profile --compare baseline.json run
```

Output shows changes in green (improvements) and red (regressions):

```text
Allocation Comparison:
Current vs Baseline
vec_growth::grow: 10 → 5 -5 (-2.50 KB)
string_builder::build: 20 → 25 +5 (+1.25 KB)
```

## Example Output

### Text Output (Default)

```text
Allocation Profile:
cargo_alloc_profile::allocator::impl$0::alloc: 51
cargo_alloc_profile::allocator::impl$0::realloc: 9
```

### With Verbosity (-v)

```text
Allocation Profile:
cargo_alloc_profile::allocator::impl$0::alloc: 51 (0.58 KB)
cargo_alloc_profile::allocator::impl$0::realloc: 9 (3.78 KB)
```

### JSON Output (-o json -v)

```json
{
  "allocations": [
    {
      "function": "cargo_alloc_profile::allocator::impl$0::alloc",
      "count": 51,
      "total_bytes": 592
    },
    {
      "function": "cargo_alloc_profile::allocator::impl$0::realloc",
      "count": 9,
      "total_bytes": 3872
    }
  ],
  "summary": {
    "total_allocations": 65,
    "total_deallocations": 10,
    "total_bytes_allocated": 6025,
    "peak_memory": 4089,
    "current_memory": 4035
  }
}
```

## Use Cases

### Finding Allocation Hotspots

```bash
# Find functions that allocate the most
cargo alloc-profile --sort-by size --limit 10 -v run
```

### Optimizing Vec Growth

Before optimization:

```rust
let mut items = Vec::new();
for i in 0..1000 {
    items.push(i);  // Multiple reallocations!
}
```

After optimization:

```rust
let mut items = Vec::with_capacity(1000);
for i in 0..1000 {
    items.push(i);  // Single allocation
}
```

Compare the difference:

```bash
cargo alloc-profile --compare before.json run
```

### Tracking Allocation Changes

```bash
# Before making changes
cargo alloc-profile --save before.json run

# ... make optimizations ...

# After changes
cargo alloc-profile --compare before.json run
```

### Integration with CI/CD

```bash
# Generate JSON report for analysis
cargo alloc-profile -o json run > allocations.json

# Set thresholds in CI
cargo alloc-profile --threshold-bytes 10000 run || echo "Large allocations detected!"
```

## How It Works

`cargo-alloc-profile` uses a custom global allocator that wraps the system allocator. When profiling is enabled:

1. Each allocation is tracked with its size and call stack
2. Allocations are aggregated by location
3. Minimal overhead using atomic operations and thread-local guards
4. Results are serialized to JSON and displayed with formatting

The profiler is careful to avoid infinite recursion - it uses thread-local reentrancy guards to prevent profiling its own allocations.

## Library Usage

You can also use the profiler directly in your code:

```rust
use cargo_alloc_profile::AllocationProfiler;

fn main() {
    AllocationProfiler::enable();
    
    // Your code here
    let data = vec![1, 2, 3, 4, 5];
    
    AllocationProfiler::write_report();
}
```

Add to your `Cargo.toml`:

```toml
[dependencies]
cargo-alloc-profile = "0.1"
```

## Performance

The profiler adds minimal overhead when enabled:

- Atomic operations for counters (no locks)
- Thread-local reentrancy detection
- Lazy backtrace resolution
- Zero overhead when disabled

## Limitations

- Currently tracks allocations at the function level
- Backtraces add some overhead (use release builds for accurate measurements)
- Does not track stack allocations (only heap via the global allocator)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Acknowledgments

Built with:

- [clap](https://github.com/clap-rs/clap) - Command line parsing
- [backtrace](https://github.com/rust-lang/backtrace-rs) - Stack trace capture
- [colored](https://github.com/colored-rs/colored) - Terminal colors
- [serde](https://github.com/serde-rs/serde) - Serialization

## See Also

- [dhat](https://github.com/nnethercote/dhat-rs) - DHAT-style heap profiling
- [heaptrack](https://github.com/KDE/heaptrack) - Heap memory profiler
- [valgrind](https://valgrind.org/) - Memory debugging and profiling tools


## See Also

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) - CPU profiling
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat) - Binary size analysis
