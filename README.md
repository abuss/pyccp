# pyccp - Python Constraint Satisfaction Problem Solver

A high-performance Python library for solving Constraint Satisfaction Problems (CSPs) and optimization problems with an efficient Rust backend.

## Latest Release

**v2.3.0** (2026-10-18) — Branch-and-Bound Optimization with `set_objective()` API

See [RELEASE_NOTES.md](RELEASE_NOTES.md) for details.

## Quick Start

```bash
pip install pyccp
```

### Basic CSP Example

```python
from pyccp import Store

# Create variables: 4 queens on 4x4 chessboard
s = Store([list(range(4)) for _ in range(4)])

# Post constraints: no two queens in same row/column/diagonal
s.post(("all_diff", (list(range(4)),)))
for i in range(4):
    for j in range(i+1, 4):
        s.post(("neq_linear", ([(i, 1), (j, -1)], j - i)))
        s.post(("neq_linear", ([(i, 1), (j, -1)], -(j - i))))

# Solve
solution = s.solve()
print(solution)  # [1, 3, 0, 2]
```

### Optimization Example

```python
from pyccp import Store

# 0/1 knapsack: select items to maximize value under weight limit
items = [(10, 5), (8, 4), (7, 3), (6, 2), (5, 1)]  # (value, weight)
s = Store([[0, 1] for _ in range(len(items))])

# Mark cost cell for optimization (v2.3+)
s.set_objective(len(items), minimize=False)

# Solve with branch-and-bound pruning
s.solve()

# Query best cost found
best_value = s.best_cost()
print(best_value)
```

## Features

- **Constraint Satisfaction**: Solve complex CSP problems with intelligent variable/value selection (MRV, LCV heuristics)
- **Optimization**: Branch-and-bound search with `set_objective()` for cost minimization/maximization (v2.3+)
- **Advanced Propagation**: AC-3 algorithm for AllDiff, bounds propagation for linear constraints
- **High Performance**: 56-62% faster propagation via lazy constraint evaluation; 20-40% speedup on optimization instances
- **Multiple Constraints**: AllDiff, Element, LinearEq, NeqLinear, Cardinality, and more
- **Production Ready**: 41 passing tests (23 CSP examples + 3 optimization examples + 15 unit tests)
- **Backward Compatible**: Drop-in replacement for v2.2 and earlier

## Documentation

- [OPTIMIZATION.md](docs/OPTIMIZATION.md) — Branch-and-bound optimization guide (v2.3+)
- [EXAMPLES.md](docs/EXAMPLES.md) — Complete list of example problems
- [CHANGELOG.md](CHANGELOG.md) — Detailed list of features and changes
- [RELEASE_NOTES.md](RELEASE_NOTES.md) — Release summary and migration guide

## Examples

See `examples/` directory for usage examples. Key examples include:

- **CSP Problems**: N-Queens, Send More Money, Graph Coloring, Magic Square, Sudoku solvers
- **Optimization**: TSP (5 cities, branch-and-bound), Knapsack (0/1 selection), Job Scheduling (makespan minimization)

## Testing

Run tests with:
```bash
pytest tests/
```

All 41 tests pass:
- 23 CSP examples
- 3 optimization examples (TSP, Knapsack, Scheduling)
- 5 optimization unit tests
- 10 other tests (agent, bridge, integration)

Rust backend: 75 tests (33 core + 7 optimization + 35 other).

## Performance

- **CSP Solving**: 5-10x speedup on hard constraint problems vs. naive search
- **Optimization**: 20-40% speedup with branch-and-bound vs. full enumeration on small instances
- **Propagation**: 56-62% faster via lazy constraint deduplication

See [PHASE1_BENCHMARK.md](PHASE1_BENCHMARK.md) and [PHASE2_BENCHMARK.md](PHASE2_BENCHMARK.md) for detailed benchmarks.

## License

See LICENSE file for details.
