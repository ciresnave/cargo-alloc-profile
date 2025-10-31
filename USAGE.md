# cargo-alloc-profile Usage Guide

## Quick Start

After installation, you can profile any Rust program:

```bash
# Install the tool
cargo install cargo-alloc-profile

# Profile your application
cargo alloc-profile run

# Profile a specific binary
cargo alloc-profile run --bin my-app

# Profile tests
cargo alloc-profile test

# Profile benchmarks
cargo alloc-profile bench
```

## How to Interpret Results

### Hot Spots Section
Shows functions with the most allocations. Look for:
- High allocation counts (>100)
- Large byte totals (>1KB per function)
- Unexpected allocations in performance-critical paths

### Per-Module Breakdown
Shows which modules allocate the most memory. Useful for:
- Identifying problematic areas of your codebase
- Comparing allocation patterns across modules
- Finding external dependencies that allocate heavily

### Zero-Allocation Functions
Functions not listed in hot spots made no heap allocations. These are optimal for performance-critical code.

## Common Optimization Patterns

### 1. Pre-allocate Collections

**Before:**
```rust
let mut v = Vec::new();
for i in 0..1000 {
    v.push(i);  // Multiple reallocations
}
```

**After:**
```rust
let mut v = Vec::with_capacity(1000);  // Single allocation
for i in 0..1000 {
    v.push(i);
}
```

**Impact:** 9 allocations → 1 allocation

### 2. Avoid Unnecessary Clones

**Before:**
```rust
fn process(data: &BigStruct) -> Result<()> {
    let copy = data.clone();  // Expensive!
    expensive_operation(&copy)
}
```

**After:**
```rust
fn process(data: &BigStruct) -> Result<()> {
    expensive_operation(data)  // Zero allocations
}
```

### 3. Use String Building Efficiently

**Before:**
```rust
let mut msg = String::new();
for item in items {
    msg = msg + &item.to_string();  // N allocations
}
```

**After:**
```rust
let capacity = items.iter().map(|i| i.to_string().len()).sum();
let mut msg = String::with_capacity(capacity);
for item in items {
    msg.push_str(&item.to_string());  // 1 allocation
}
```

### 4. Use References Instead of Owned Values

**Before:**
```rust
fn format_all(items: Vec<String>) -> Vec<String> {
    items.into_iter()
        .map(|s| s.to_uppercase())  // Allocates for each
        .collect()
}
```

**After:**
```rust
fn format_all(items: &[String]) -> Vec<String> {
    items.iter()
        .map(|s| s.to_uppercase())
        .collect()
}
```

## Advanced Usage

### Integration with CI/CD

You can fail builds if allocations exceed a threshold:

```bash
cargo alloc-profile test 2>&1 | tee alloc-report.txt
# Parse the report and fail if allocations > threshold
```

### Comparing Before/After

```bash
# Baseline
cargo alloc-profile run > before.txt

# After optimization
cargo alloc-profile run > after.txt

# Compare
diff before.txt after.txt
```

### Profiling Specific Tests

```bash
# Profile a single test
cargo alloc-profile test test_name

# Profile tests matching a pattern
cargo alloc-profile test -- integration
```

## Performance Considerations

- **Profiling overhead:** 10-50x slowdown due to backtrace capture
- **Only for development:** Never use in production
- **Best for:** Finding allocation hot spots, not measuring absolute performance

## Troubleshooting

### No profiling output
Make sure you're running through `cargo alloc-profile`, not `cargo run` directly.

### Missing symbols
Build with debug symbols enabled:
```toml
[profile.release]
debug = true
```

### Too much output
Focus on specific modules:
```bash
cargo alloc-profile test my_module
```

## Examples

See the `examples/` directory for demonstrations:
- `demo.rs` - Comprehensive allocation patterns
- `simple.rs` - Basic usage example

Run examples with:
```bash
cargo run --example demo
cargo run --example simple
```

## Contributing

Found a bug or have a feature request? Please open an issue or submit a PR!

## License

MIT OR Apache-2.0
