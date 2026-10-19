# pyccp - Python Constraint Satisfaction Problem Solver

A high-performance Python library for solving Constraint Satisfaction Problems (CSPs) with an efficient Rust backend.

## Latest Release

**v2.1.0** (2026-10-18) — CSP Framework Upgrade with 5-10x theoretical speedup

See [RELEASE_NOTES.md](RELEASE_NOTES.md) for details.

## Quick Start

```bash
pip install pyccp
```

## Features

- **Efficient CSP Solving**: Intelligent variable and value selection heuristics (MRV, LCV)
- **Advanced Propagation**: AC-3 algorithm for AllDiff, bounds propagation for linear constraints
- **High Performance**: 56-62% faster propagation via lazy constraint evaluation
- **Production Ready**: Comprehensive test suite with 88 passing tests
- **Backward Compatible**: Drop-in replacement for previous versions

## Documentation

- [CHANGELOG.md](CHANGELOG.md) — Detailed list of features and changes
- [RELEASE_NOTES.md](RELEASE_NOTES.md) — Release summary and migration guide
- [REGRESSION_TEST_REPORT.md](REGRESSION_TEST_REPORT.md) — Full test results
- [PHASE1_BENCHMARK.md](PHASE1_BENCHMARK.md) — Propagation benchmarks
- [PHASE2_BENCHMARK.md](PHASE2_BENCHMARK.md) — Search and heuristic benchmarks

## Examples

See `examples/` directory for usage examples.

## Testing

Run tests with:
```bash
pytest tests/
```

All 88 tests pass:
- 33 Rust library tests
- 24 integration tests
- 31 Python tests

## License

See LICENSE file for details.
