# Changelog - pyccp

## [2.1.0] - 2026-10-18

### Added

- **Bitset domain representation** for faster domain operations with O(1) membership checks
- **MRV (Minimum Remaining Values) heuristic** for intelligent variable selection during search
- **LCV (Least Constraining Value) heuristic** for value ordering that reduces future conflicts
- **Bounds tracking (min/max)** per variable for efficient constraint propagation
- **Bounds propagation** for LinearEq and LessThan constraints with automatic tightening
- **AC-3 algorithm** for AllDiff constraint with efficient arc consistency
- **Lazy propagation queue** to skip unaffected constraints and reduce overhead

### Changed

- **Performance optimizations**: 56-62% improvement in propagation overhead (See [PHASE1_BENCHMARK.md](PHASE1_BENCHMARK.md) and [PHASE2_BENCHMARK.md](PHASE2_BENCHMARK.md))
- **Improved search efficiency** with heuristic-guided variable and value selection
- Propagation now uses deduplication to eliminate redundant constraint checks

### Fixed

- Fixed propagation queue overhead with Vec-based deduplication strategy

### Testing

See [REGRESSION_TEST_REPORT.md](REGRESSION_TEST_REPORT.md) for comprehensive test results (88 total tests: 33 Rust library + 24 integration + 31 Python).

### Performance Metrics

Refer to benchmark reports for detailed performance analysis:
- [PHASE1_BENCHMARK.md](PHASE1_BENCHMARK.md) - Core propagation benchmarks
- [PHASE2_BENCHMARK.md](PHASE2_BENCHMARK.md) - Search and heuristic benchmarks

---

## [0.2.0] - Previous Release

Baseline implementation with core CSP solving capabilities.
