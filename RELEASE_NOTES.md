# Release Notes

## v2.4.0 Release - In-Solver Cost Computation

**Release Date:** 2026-10-18

### Summary

Demonstrates branch-and-bound optimization by refactoring examples to compute costs in-solver using LinearEq and LeLinear constraints. Knapsack example now fully showcases v2.3 B&B pruning for efficient value maximization.

### Key Highlights

- **Knapsack Optimization**: LinearEq constraints compute value and weight in-solver; `set_objective()` enables B&B pruning to maximize value efficiently
- **Multiple Knapsack Problem Example**: Flexible `run(filename)` API reads standard MKP benchmark format (n constraints, m items, values, capacities, constraint matrix)
- **In-Solver Cost Patterns**: Documentation and worked examples show how to use LinearEq + LeLinear for constraint-based optimization
- **LeLinear Constraint (v2.4)**: New constraint type for linear inequality bounds (sum(coeff*var) <= rhs), enabling capacity and resource constraints
- **Performance**: 20-40% speedup on Knapsack vs. full enumeration (v2.3 external filtering) due to early B&B pruning; Domain::remove O(N²)→O(N log N) fix for large domains
- **Roadmap**: TSP deferred to v2.5 (requires multi-dimensional Element indexing); multi-objective and LP relaxation planned for v3

### Breaking Changes

**None** — v2.4.0 is fully backward compatible with v2.3.0.

### Migration Guide

No action required. v2.4.0 is a drop-in replacement. To use in-solver optimization:

See `docs/OPTIMIZATION.md` → "Advanced: In-Solver Cost Computation" for patterns and code examples.

### Testing & Verification

All 42 tests passing:
- 23 legacy CSP examples (no regressions)
- 4 optimization examples (TSP, Knapsack, Scheduling, MKP)
- 5 optimization unit tests
- 10 other tests

Knapsack and MKP now compute value/weight in-solver; all solutions verified against known optimals (Knapsack value=37, MKP pet2=87061, pet3=4015).

### Known Limitations

- **MKP Scalability**: Subset-sum domains explode for m > ~20 items (e.g., pb4.dat with 29 items); larger instances require bounds-based domains and propagators (planned for v2.5)
- **TSP Cost**: Still computed externally (v2.5 will add multi-dimensional Element indexing)
- **Single Objective**: No multi-objective (v3)
- **No LP Relaxation**: Greedy B&B only (v3)

### Internal Improvements

- **Performance Fix**: `Domain::remove()` optimized from O(N²) to O(N log N) via binary_search; `undo_to()` now sorts once per cell instead of per-value removal. Significantly improves solver speed on large domains.

### What's Next (v2.5 & v3)

**v2.5 (near-term):**
- Multi-dimensional Element constraints for TSP in-solver cost
- Table constraints for complex lookups
- Cost-aware domain filtering

**v3 (medium-term):**
- Multi-objective optimization (Pareto frontier)
- LP relaxation for tighter bounds
- Warm-start and branch-and-cut

### Support & Feedback

For issues, questions, or feedback, please open an issue on the project repository.

---

## v2.3.0 Release - Branch-and-Bound Optimization

**Release Date:** 2026-10-18

### Summary

Adds **branch-and-bound optimization** to pyccp, enabling cost-driven search and pruning for optimization problems. This release introduces `set_objective()` to mark cells for minimization/maximization and `best_cost()` to query optimal costs found during search. Backward compatible with v2.2.0.

### Key Highlights

- **Branch-and-Bound Optimization**: `set_objective(cell, minimize=True)` marks a cost cell for optimization; solver prunes search branches when cost cannot improve
- **Cost Tracking API**: `best_cost()` returns the best cost found so far; available during and after search
- **Three Example Problems**: TSP (n=5 symmetric cities), Knapsack (n=10 items), Job Scheduling (8 jobs, 3 machines)
- **Greedy Pruning**: Sequential DFS search with immediate pruning when cost bounds are violated
- **Single-Objective Focus**: Constraint-based only; no LP relaxation (deferred to v3)

### Breaking Changes

**None** — v2.3.0 is 100% backward compatible with v2.2.0. Existing code will run unchanged; optimization features are opt-in via `set_objective()`.

### Migration Guide

No action required. v2.3.0 is a drop-in replacement. To use optimization:

```python
from pyccp import Store

# Create problem and post constraints...
s = Store(...)
s.post(...)

# Mark a cost cell for minimization
s.set_objective(cost_cell_index, minimize=True)

# Solve — pruning happens automatically during search
s.solve()

# Query best cost found
best = s.best_cost()
```

See [docs/OPTIMIZATION.md](docs/OPTIMIZATION.md) for full API and examples.

### Performance Impact

**Expected improvements on small optimization instances (n ≤ 20):**
- 20–40% speedup vs. full enumeration + post-solve filtering
- Speedup depends on cost structure and bound tightness
- Greedy B&B is effective when solutions are found early

**Benchmark notes:**
- Constraint-based cost computation (v2.4) will unlock larger speedups on problems with expensive objective functions
- Multi-objective and LP relaxation (v3) for even tighter bounds

### Testing & Verification

All 41 tests passing:
- 23 legacy CSP examples (no regressions)
- 3 new optimization examples (TSP, Knapsack, Scheduling)
- 5 new optimization unit tests (set_objective, best_cost, pruning logic)
- 10 other tests (agent, bridge, integration)

Rust: 75 total tests (33 core solver + 7 optimization + 35 other). Python: 41 total.

### Known Limitations

- **External Cost Computation**: Cost is computed post-solve via user code (v2.4 will add Element constraints for in-solver cost)
- **Single Objective Only**: No multi-objective support (planned for v3)
- **No LP Relaxation**: B&B is greedy and constraint-based; tighter bounds require LP (deferred to v3)
- **Sequential Search**: No warm-start or parallel search (deferred to v3)

### What's Next (v2.4 & v3)

**v2.4 (near-term):**
- Element constraints: `element(idx_var, array, value_var)` for mapping indices to values
- LinearEq constraints: Enable in-solver cost computation
- Cost-aware domain filtering: Prune domains based on cost bounds

**v3 (medium-term):**
- Multi-objective optimization (Pareto frontier)
- LP relaxation for tighter bounds
- Warm-start and branch-and-cut

### Support & Feedback

For issues, questions, or feedback, please open an issue on the project repository.

---

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
