# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-31

### Added

- Initial release of cargo-alloc-profile
- Core allocation profiling functionality
- Custom global allocator with backtrace capture
- CLI commands: `run`, `test`, `bench`
- Multiple verbosity levels (`-v`, `-vv`, `-vvv`)
- Filtering options:
  - `--filter` to filter by function name
  - `--min-count` to show allocations above threshold
  - `--threshold-bytes` to show allocations over byte limit
- Sorting options: `--sort-by count|size|name`
- Grouping options: `--group-by function|module|file`
- Output formats: text (default) and JSON (`-o json`)
- `--limit` option to show only top N allocation sites
- `--save` option to save profiling data for comparison
- `--compare` option to compare runs and show deltas
- Thread-local reentrancy guards to prevent infinite recursion
- Atomic operations for lock-free profiling
- Colored terminal output
- Comprehensive README with usage examples
- MIT/Apache-2.0 dual licensing
- Example programs demonstrating usage

### Fixed

- Stack overflow protection with reentrancy detection
- Accurate memory tracking (deallocations properly counted)

[0.1.0]: https://github.com/ciresnave/cargo-alloc-profile/releases/tag/v0.1.0
