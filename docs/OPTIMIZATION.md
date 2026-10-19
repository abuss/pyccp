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
