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
