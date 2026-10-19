# Optimization with pyccp v2.3

## Overview

v2.3 adds **branch-and-bound optimization** to pyccp. Instead of finding any feasible solution, you can now find the *best* solution under a cost metric.

## How It Works

1. **Define an objective cell**: One variable in your problem represents the cost/value to optimize.
2. **Set the objective**: Tell pyccp to minimize or maximize that cell.
3. **Solve**: pyccp explores the search tree, pruning branches that can't improve the best solution found so far.

## API

### Setting an Objective

```python
from pyccp import Store

s = Store(domains)
s.post(...)  # post constraints
s.set_objective(cell=cost_cell_index, minimize=True)  # minimize cost
sol = s.solve()  # returns optimal solution
```

### Checking Best Cost

```python
best = s.best_cost()  # None if no solution yet; otherwise the best cost found
```

## Example: TSP (Travelling Salesman Problem)

Find the shortest tour through n cities.

```python
from pyccp import Store

dist = [
    [0, 2, 3],
    [2, 0, 1],
    [3, 1, 0],
]
n = len(dist)

# Variables: tour[i] = next city after city i
s = Store([list(range(n)) for _ in range(n)])

# Constraint: visit each city exactly once
s.post(("all_diff", (list(range(n)),)))

# Enumerate all permutations
sols = s.solve_all()

# Find the one with minimum total distance
best_cost = float('inf')
best_tour = None
for tour in sols:
    cost = sum(dist[i][tour[i]] for i in range(n))
    if cost < best_cost:
        best_cost = cost
        best_tour = tour

print(f"Optimal tour: {best_tour}, cost: {best_cost}")
```

## Example: Knapsack (0/1 Knapsack Problem)

Pack items into a knapsack of limited capacity to maximize total value.

```python
from pyccp import Store

# Items: (value, weight)
items = [
    (10, 5),   # item 0
    (8, 4),    # item 1
    (7, 3),    # item 2
    (6, 2),    # item 3
    (5, 1),    # item 4
]
n = len(items)
capacity = 12

# Variables: selected[i] in {0, 1} — item i included or not
s = Store([[0, 1] for _ in range(n)])

# Enumerate all 2^n combinations
sols = s.solve_all(limit=2**n)

# Find maximum value under capacity constraint
best_val = 0
best_sel = None
for sol in sols:
    total_weight = sum(sol[i] * items[i][1] for i in range(n))
    if total_weight <= capacity:
        total_value = sum(sol[i] * items[i][0] for i in range(n))
        if total_value > best_val:
            best_val = total_value
            best_sel = sol

print(f"Selected items: {best_sel}, value: {best_val}")
```

## Example: Scheduling (Job Scheduling)

Assign jobs to machines to minimize makespan (total completion time).

```python
from pyccp import Store

job_durations = [5, 3, 4, 6, 2, 7, 3, 1]  # 8 jobs
n_jobs = len(job_durations)
n_machines = 3

# Variables: machine[i] in 0..n_machines-1 — job i assigned to which machine
s = Store([list(range(n_machines)) for _ in range(n_jobs)])

# Enumerate all possible assignments
sols = s.solve_all(limit=3**n_jobs)

# Find assignment with minimum makespan
best_makespan = float('inf')
best_assign = None
for sol in sols:
    machine_times = [0] * n_machines
    for job_idx, machine_idx in enumerate(sol):
        machine_times[machine_idx] += job_durations[job_idx]
    makespan = max(machine_times)
    if makespan < best_makespan:
        best_makespan = makespan
        best_assign = sol

print(f"Job assignment: {best_assign}, makespan: {best_makespan}")
```

## Advanced: In-Solver Cost Computation (v2.4)

### Overview

v2.3 marked cost cells for optimization but computed costs externally (post-solve).
v2.4 introduces in-solver cost computation using LinearEq and LeLinear constraints,
enabling the solver to prune search branches based on cost bounds during optimization.

### Pattern: Knapsack Value Maximization

The 0/1 knapsack problem demonstrates the complete in-solver optimization pattern:

```python
from pyccp import Store

items = [(10, 5), (8, 4), (7, 3), (6, 2), (5, 1)]  # (value, weight)
capacity = 12

# Variables: selected[i] ∈ {0,1}, value, weight
n = len(items)
s = Store([[0, 1] for _ in range(n)] +
          [list(range(50))] +  # value cell
          [list(range(50))])   # weight cell

# In-solver cost computation via LinearEq
value_idx = n
weight_idx = n + 1

# value = sum(selected[i] * items[i].value)
value_terms = [(i, items[i][0]) for i in range(n)] + [(value_idx, -1)]
s.post(("linear_eq", (value_terms, 0)))

# weight = sum(selected[i] * items[i].weight)
weight_terms = [(i, items[i][1]) for i in range(n)] + [(weight_idx, -1)]
s.post(("linear_eq", (weight_terms, 0)))

# Capacity constraint: weight <= capacity
s.post(("le_linear", ([(i, items[i][1]) for i in range(n)], capacity)))

# Optimization: maximize value
s.set_objective(value_idx, minimize=False)
s.solve_all()

best_value = s.best_cost()
print(f"Maximum value: {best_value}")
```

### Key Points

1. **LinearEq Constraint:** Maps variable values to computed cells
   - Syntax: `("linear_eq", ([(var_idx, coeff), ..., (result_cell, -1)], 0))`
   - Semantics: `sum(cell[var_idx] * coeff) = 0` (right-hand side is always 0)
   - Result cell = sum of (selected variable × coefficient)

2. **LeLinear Constraint (v2.4):** Enforces linear inequality bounds
   - Syntax: `("le_linear", ([(var_idx, coeff), ...], rhs))`
   - Semantics: `sum(cell[var_idx] * coeff) <= rhs`
   - Example: capacity constraint on total weight

3. **Objective Cell:** The cell passed to `set_objective()` must be computed by constraints
   - Solver fixes the objective cell value during search
   - B&B prunes when the cell cannot improve past `best_cost()`

4. **Search Efficiency:** In-solver cost computation enables pruning
   - Without constraints: solver enumerates all 2^n solutions, scores externally
   - With constraints: solver prunes branches early when bounds are violated
   - Expected speedup: 20-40% on small instances (n ≤ 20)

### Pattern: TSP Edge Costs (v2.5+)

Traveling Salesman requires 2D indexing: edge cost = dist[from_city][to_city].

**v2.5 (NOW AVAILABLE):** Element2D constraint enables dynamic 2D array indexing. See `examples/tsp.py` for the complete implementation:
- Element constraints map tour[i] → cost[i] via distance matrix rows
- LinearEq constraint sums costs to total_cost variable
- set_objective(minimize total_cost) enables automatic B&B pruning

This demonstrates the impact of in-solver cost computation: solver prunes branches that exceed best_cost, yielding 10-30% speedup vs. full enumeration.

### Limitations & Roadmap

- **Single Objective:** v2.5 supports one cost cell only (multi-objective in v3)
- **Linear Costs:** Objective must be expressible via LinearEq (non-linear objectives in v3)
- **No LP Relaxation:** B&B is greedy constraint-based (LP relaxation in v3)
- **Element2D Only:** 3D+ indexing deferred to v3

See RELEASE_NOTES.md for the full v3 roadmap.

## Advanced: Bounds-Based Domains for Large Problems (v2.5.1)

### Overview

v2.5.1 introduces `Domain::Bounds(min, max)` for large-capacity constraint problems. Instead of enumerating all values in a domain, Bounds stores only the interval boundaries, enabling O(1) space per domain regardless of capacity.

**Typical use case:** Multiple Knapsack Problem (MKP) with large capacities. A single knapsack with capacity 55,000 requires a domain of 55,001 values — prohibitive in memory. Bounds represents this as O(1) interval [0, 55000].

### When to Use Bounds

Use Bounds-based domains when:

1. **Domain size > 1000 values**: Memory becomes a bottleneck
2. **Large capacity constraints**: Subset-sum, knapsack, or bin-packing problems
3. **Tight coupling (many constraints)**: Bounds propagation is effective across linear constraints
4. **Optimizing for runtime/memory**: Accept slightly less precise conflict detection for massive speedup

Example problems: MKP with m ≥ 25 items, bin packing with 1000+ bin capacity, scheduling with large time horizons.

### How It Works (Transparent API)

**Good news:** No API changes. `Store::new()` automatically converts:

```python
from pyccp import Store

# Example: MKP with large capacity (55,000)
# Capacity domain would be 55,001 values if enumerated
capacity_domain = list(range(55001))  # Huge!

s = Store([capacity_domain, ...])  # Store::new() auto-converts
# Internally: capacity uses Domain::Bounds(0, 55000) — O(1) memory
# Smaller domains remain Vec for precision
```

**Detection:** Store inspects domain size at creation:
- Domain > 1000 values → Auto-convert to Bounds
- Domain ≤ 1000 values → Keep as Vec (full enumeration)

### Performance Impact

| Scenario | v2.4 | v2.5.1 | Benefit |
|----------|------|--------|---------|
| MKP pb4.dat (m=29, capacity=55000) | Out of Memory | <30s | ✅ Feasible |
| Memory per capacity domain | 55,001 ints × 8B = 440KB | O(1) = 16B | ~27,500x |
| Search pruning (B&B) | Fewer branches cut early | Same B&B logic | No regression |

### Known Limitations

1. **AllDiff Sparse Domains**: Bounds-based AllDiff cannot detect inconsistencies in sparse domains (e.g., {1, 3, 5, 7}). Acceptable trade-off — use AllDiff only on compact ranges or combined with other constraints.
   
2. **Threshold Tuning**: The 1000-value threshold may need adjustment based on your memory budget. Mark with a config constant if you need to tune.

3. **Ceiling (Roadmap)**: Current B&B may saturate at m~35-40 items. Larger MKP instances benefit from LP relaxation (v3).

### Example: MKP with Bounds Domains

See [examples/mknapsack/mknapsack.py](../examples/mknapsack/mknapsack.py) for the full MKP solver, which automatically uses Bounds for large capacities.

For performance analysis and benchmark results, see [BENCHMARK_REPORT_v2.5.1.md](../BENCHMARK_REPORT_v2.5.1.md).

### Limitations & Roadmap

- **Single Objective:** v2.5 supports one cost cell only (multi-objective in v3)
- **Linear Costs:** Objective must be expressible via LinearEq (non-linear objectives in v3)
- **No LP Relaxation:** B&B is greedy constraint-based (LP relaxation in v3)
- **Element2D Only:** 3D+ indexing deferred to v3

See RELEASE_NOTES.md for the full v3 roadmap.

## Limitations

- **Single objective only**: Multi-objective optimization is not supported.
- **No LP relaxation**: Bounds are purely constraint-based; no continuous relaxation.
- **Sequential search**: DFS exploration; no parallel branch-and-bound yet.
- **Greedy pruning**: Current best cost updates greedily; no lookahead or advanced bounding.

## Performance

Branch-and-bound can significantly reduce search time on problems with:
- **Large search spaces**: Pruning eliminates many branches.
- **Strong cost structure**: Objective provides tight bounds early.

Typical speedups: 20–40% on small TSP/Knapsack instances.

## Future Work

- **Multi-objective**: Pareto frontier enumeration.
- **Warm-start**: Accept initial solution to warm the bound.
- **Advanced bounding**: LP relaxation for tighter bounds.
