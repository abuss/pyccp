# Release Notes

## v2.1.0 Release - CSP Framework Upgrade

**Release Date:** 2026-10-18

### Summary

Major optimization release with **5-10x theoretical speedup** on hard constraint satisfaction problems. This release focuses on intelligent search strategies and efficient constraint propagation to dramatically improve solver performance while maintaining 100% backward compatibility.

### Key Highlights

- **Heuristic-Driven Search**: MRV (Minimum Remaining Values) and LCV (Least Constraining Value) heuristics intelligently guide variable and value selection during search, reducing the search space exponentially on hard problems
- **Advanced Propagation**: Bitset-based domain representation and AC-3 algorithm for AllDiff, with bounds propagation for linear constraints
- **Lazy Constraint Evaluation**: Skip unaffected constraints via deduplication, reducing propagation overhead by 56-62%
- **Performance Metrics**: Comprehensive benchmarks show significant improvements across diverse CSP instances

### Breaking Changes

**None** — v2.1.0 is 100% backward compatible with v0.2.0. All existing code will run without modification.

### Migration Guide

No action required. v2.1.0 is a drop-in replacement for v0.2.0. Simply upgrade and enjoy improved performance.

```bash
pip install --upgrade pyccp
```

### Performance Impact

Detailed performance metrics are available in:
- [PHASE1_BENCHMARK.md](PHASE1_BENCHMARK.md) — Core propagation performance (56-62% improvement)
- [PHASE2_BENCHMARK.md](PHASE2_BENCHMARK.md) — Search efficiency and heuristic impact

**Expected improvements:**
- Hard CSP instances: 5-10x theoretical speedup
- Medium complexity: 2-3x typical improvement
- Small instances: May see lazy queue overhead (minimal, <5% regression) — see benchmarks for details

### Testing & Verification

All 88 tests passing (33 Rust library + 24 integration + 31 Python). Comprehensive regression testing confirms no behavioral regressions.

See [REGRESSION_TEST_REPORT.md](REGRESSION_TEST_REPORT.md) for full test results and coverage analysis.

### Known Limitations

- **Lazy Queue Overhead**: On very small problems (<50 variables), the lazy queue deduplication overhead may slightly reduce performance (~5% regression). This is negligible compared to gains on realistic instances.
- See benchmark reports for performance profiles by problem size and complexity.

### What's Next

Post-release optimization opportunities are documented in:
- `docs/superpowers/plans/` — Future optimization roadmap

### Support & Feedback

For issues, questions, or feedback, please open an issue on the project repository.

---

### Detailed Change Log

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of all features, improvements, and fixes in this release.
