# Project Structure

```
cargo-alloc-profile/
├── Cargo.toml              # Package manifest with dependencies
├── Cargo.lock              # Dependency lock file
├── README.md               # Main documentation
├── USAGE.md                # Detailed usage guide
├── .gitignore              # Git ignore patterns
├── src/
│   ├── main.rs             # CLI entry point (cargo subcommand)
│   ├── lib.rs              # Library exports
│   ├── allocator.rs        # Global allocator with profiling
│   ├── profiler.rs         # Allocation tracking and data collection
│   └── reporter.rs         # Report generation and formatting
├── examples/
│   ├── demo.rs             # Comprehensive demonstration
│   └── simple.rs           # Simple usage example
└── target/                 # Build artifacts
```

## Key Components

### 1. CLI (main.rs)
- Implements cargo subcommand interface
- Handles `cargo alloc-profile run/test/bench` commands
- Sets `CARGO_ALLOC_PROFILE=1` environment variable

### 2. Allocator (allocator.rs)
- Custom global allocator wrapping System allocator
- Captures backtraces on each allocation
- Only active when `CARGO_ALLOC_PROFILE=1` is set

### 3. Profiler (profiler.rs)
- Collects allocation statistics
- Aggregates allocations by call site
- Resolves and cleans symbol names
- Thread-safe using atomic operations and mutexes

### 4. Reporter (reporter.rs)
- Generates colored, formatted reports
- Shows hot spots, module breakdown, and summary
- Highlights optimization opportunities

## Installation

```bash
# From local directory
cargo install --path .

# The binary will be installed to ~/.cargo/bin/cargo-alloc-profile
```

## Usage Examples

```bash
# Profile the demo example
cargo run --example demo

# Profile a simple test
cargo run --example simple

# After installation, use as cargo subcommand
cargo alloc-profile run --example demo
```

## Development Status

✅ Core functionality implemented
✅ CLI interface working
✅ Allocation tracking with backtraces
✅ Report generation
✅ Examples created
✅ Documentation written

## Next Steps (Future Enhancements)

- [ ] Add JSON output format for CI/CD integration
- [ ] Implement allocation flamegraphs
- [ ] Add filtering by module/crate
- [ ] Create interactive HTML reports
- [ ] Add comparison mode (before/after)
- [ ] Implement allocation lifecycle tracking (alloc + dealloc pairing)
- [ ] Add memory leak detection
- [ ] Support for custom allocators

## Known Limitations

1. **Performance:** 10-50x slowdown due to backtrace capture
2. **Platform:** Backtrace quality varies by platform and debug symbols
3. **Overhead:** Not suitable for production use
4. **Accuracy:** Some allocations may be attributed to wrong functions due to inlining

## Technical Details

- **Global Allocator:** Uses `#[global_allocator]` to intercept all allocations
- **Backtrace Capture:** Uses `backtrace` crate for stack traces
- **Thread Safety:** Atomic counters + mutex-protected hash map
- **Symbol Resolution:** Cleans up Rust mangled names and removes hash suffixes
- **CLI Parsing:** Uses `clap` with derive macros

## Contributing

To contribute:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

Dual-licensed under MIT OR Apache-2.0 (choose either)
