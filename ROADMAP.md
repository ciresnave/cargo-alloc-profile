# Roadmap

This document outlines potential future features and enhancements for `cargo-alloc-profile`.

## Future Features

### Timeline Visualization

Track timestamps for each allocation to create timeline views of memory usage patterns. This would show when allocations happen during program execution.

**Trade-offs:**

- Would require storing timestamps for each allocation
- Additional overhead during profiling
- Could be toggleable to avoid impact when not needed

### Flamegraph Integration

Export profiling data in flamegraph format to visualize allocation call stacks using existing flamegraph tools.

**Benefits:**

- Leverage mature visualization tools
- Standard format for performance analysis
- Easy to share and analyze

### HTML Report

Generate interactive HTML reports with charts and graphs showing allocation patterns, memory usage over time, and hotspots.

**Potential libraries:**

- `plotters` for chart generation
- Interactive filtering and sorting
- Export/share capability

### Watch Mode

Continuously profile applications and update the display in real-time, similar to `cargo watch`.

**Use cases:**

- Development workflow integration
- Live monitoring of allocation behavior
- Immediate feedback during optimization

### Memory Leak Detection

Track allocations that are never freed to identify potential memory leaks.

**Features:**

- Identify long-lived allocations
- Report allocations still alive at program exit
- Threshold-based warnings
- Integration with existing leak detection tools

## Contributing

If you're interested in implementing any of these features or have other ideas, please open an issue or pull request on GitHub!

## Priority

Features are listed in no particular order. Priority will be determined based on:

- Community feedback and requests
- Implementation complexity
- Performance impact
- Maintenance burden

## Version Planning

These features are aspirational and not committed to any specific version. The v0.1.x series will focus on stability and bug fixes before adding major new features.
