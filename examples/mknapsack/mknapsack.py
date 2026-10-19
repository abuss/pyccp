"""Multiple Knapsack Problem (MKP) solver using pyccp.

The MKP generalizes the 0/1 knapsack to m items and n resource constraints
(knapsacks). Each item has a value and a weight for each constraint.
Goal: maximize total value subject to all capacity constraints.

Format: Read MKP instances from standard benchmark data files.
Data structure (from examples/mknapsack/data/readme):
  Line 1:   n m          # n knapsacks, m items
  Line 2:   values       # m item values
  Line 3:   capacities   # n knapsack capacity limits
  Lines 4+: constraints  # n×m matrix (constraint matrix A)
  Line n+4: optimum      # known optimal value
  Line n+5: name         # problem name
  Line n+6: ok (optional)

Example (pet2.dat):
  10  10                           # 10 constraints, 10 items
  6001 3105 18000 ... 3270        # 10 values
  450  540  200 ... 480           # 10 capacities
  20   5    100 ... 40            # constraint 1: weights of items
  20   7    130 ... 40            # constraint 2: weights of items
  ...
  87061                            # known optimum
  PETERSEN 2                       # problem name

v2.5.1 Bounds Optimization:
  Large capacity domains (>1000 values) are automatically converted to 
  compact Bounds representation in Store::new(). This reduces memory usage
  and speeds up propagation significantly for large problems like pb4.dat.
  The conversion is transparent to the API - no changes to constraint posting.
"""

import os
from pathlib import Path
from pyccp import Store


def parse_mknapsack_file(filename):
    """Parse a Multiple Knapsack Problem data file.
    
    Args:
        filename: Path to .dat file
        
    Returns:
        dict with keys:
        - n_constraints: number of knapsacks/constraints
        - m_items: number of items
        - values: list of m item values
        - capacities: list of n capacity limits
        - constraint_matrix: list of n constraint vectors (each has m weights)
        - optimum: known optimal value from file
        - name: problem name
    """
    with open(filename, 'r') as f:
        lines = [line.strip() for line in f.readlines() if line.strip()]
    
    # Parse header: n m
    header = lines[0].split()
    n_constraints = int(header[0])
    m_items = int(header[1])
    
    line_idx = 1
    
    # Parse values (m values, may span multiple lines)
    values = []
    while len(values) < m_items:
        value_line = lines[line_idx].split()
        values.extend([int(v) for v in value_line])
        line_idx += 1
    values = values[:m_items]
    
    # Parse capacities (n values, may span multiple lines)
    capacities = []
    while len(capacities) < n_constraints:
        capacity_line = lines[line_idx].split()
        capacities.extend([int(c) for c in capacity_line])
        line_idx += 1
    capacities = capacities[:n_constraints]
    
    # Parse constraint matrix (n×m matrix, may span multiple lines)
    constraint_matrix = []
    for k in range(n_constraints):
        constraint = []
        while len(constraint) < m_items:
            constraint_line = lines[line_idx].split()
            constraint.extend([int(w) for w in constraint_line])
            line_idx += 1
        constraint = constraint[:m_items]
        constraint_matrix.append(constraint)
    
    # Parse optimum value
    optimum = int(lines[line_idx])
    line_idx += 1
    
    # Parse name
    name = lines[line_idx] if line_idx < len(lines) else "unknown"
    
    return {
        'n_constraints': n_constraints,
        'm_items': m_items,
        'values': values,
        'capacities': capacities,
        'constraint_matrix': constraint_matrix,
        'optimum': optimum,
        'name': name,
    }


def run(filename=None):
    """Solve a Multiple Knapsack Problem instance.
    
    Args:
        filename: Path to MKP data file. If None, uses default (pet2.dat).
        
    Returns:
        tuple (selected, total_value) or None if no solution found
        - selected: list of binary selections (1 = item selected, 0 = not)
        - total_value: total value of selected items
    """
    if filename is None:
        # Default to pet2.dat in the same directory
        base_dir = Path(__file__).parent
        filename = base_dir / 'data' / 'pet2.dat'
    
    filename = str(filename)
    
    # Parse the data file
    data = parse_mknapsack_file(filename)
    
    n = data['n_constraints']  # number of knapsacks/constraints
    m = data['m_items']        # number of items
    values = data['values']
    capacities = data['capacities']
    constraint_matrix = data['constraint_matrix']
    optimum = data['optimum']
    name = data['name']
    
    print(f"Solving: {name}")
    print(f"  Items: {m}, Constraints: {n}")
    print(f"  Optimum (known): {optimum}")
    print(f"  Capacities: {capacities}")
    print()
    
    # Variables: selected[0..m-1] ∈ {0,1}, computed[0..n-1] (constraint sums), value
    # Cell layout: selected=0..m-1, computed=m..m+n-1, value_cell=m+n
    computed_start = m
    value_cell = m + n

    # Domains as achievable subset sums, not full ranges. Tight domains keep
    # propagation cheap; full ranges make the propagate loop O(2^m) per leaf.
    # ponytail: subset-sum domains explode for m > ~20 (e.g. pb4.dat m=29).
    # Upgrade path: bounds-based domains + bound-tightening propagators so
    # is_fixed uses min==max instead of enumerated domains.
    def subset_sums(coeffs):
        sums = {0}
        for c in coeffs:
            sums |= {s + c for s in sums}
        return sorted(sums)

    domains = (
        [[0, 1] for _ in range(m)] +
        [subset_sums(constraint_matrix[k]) for k in range(n)] +
        [subset_sums(values)]
    )
    
    s = Store(domains)
    
    # Constraint: For each knapsack k, compute the sum of weights
    # sum(selected[i] * constraint_matrix[k][i]) = computed[k]
    for k in range(n):
        terms = [(i, constraint_matrix[k][i]) for i in range(m)] + [(computed_start + k, -1)]
        s.post(("linear_eq", (terms, 0)))
    
    # Constraint: For each knapsack k, respect capacity
    # computed[k] <= capacities[k]
    for k in range(n):
        s.post(("le_linear", ([(computed_start + k, 1)], capacities[k])))
    
    # Constraint: compute total value
    # sum(selected[i] * values[i]) = value_cell
    value_terms = [(i, values[i]) for i in range(m)] + [(value_cell, -1)]
    s.post(("linear_eq", (value_terms, 0)))
    
    # Objective: maximize value
    s.set_objective(value_cell, minimize=False)
    
    # Solve with branch-and-bound
    # solve_all() with a reasonable limit finds the best solution via B&B pruning.
    # Limit is conservative; B&B terminates early once the optimum is proven.
    print("Solving with B&B...")
    all_solutions = s.solve_all(limit=1000)
    
    if not all_solutions and s.best_cost() is None:
        print(f"{name}: no solution found")
        return None
    
    best = s.best_cost()
    if best is None:
        print(f"{name}: no solution found")
        return None
    
    # Find solution with optimal value (best_cost from B&B)
    solution = None
    for sol in all_solutions:
        if sol[value_cell] == best:
            solution = sol
            break
    
    # If no solution in all_solutions, use first (shouldn't happen)
    if not solution and all_solutions:
        solution = all_solutions[0]
    
    # Extract results
    selected = solution[:m]
    total_value = sum(selected[i] * values[i] for i in range(m))
    
    # Verify constraints
    constraint_values = [sum(selected[i] * constraint_matrix[k][i] for i in range(m)) for k in range(n)]
    feasible = all(constraint_values[k] <= capacities[k] for k in range(n))
    
    print(f"Solution found:")
    print(f"  Total value: {total_value} (vs optimum {optimum})")
    print(f"  Gap: {optimum - total_value} ({100*(optimum-total_value)/optimum:.1f}%)")
    print(f"  Constraint values: {constraint_values}")
    print(f"  Capacities: {capacities}")
    print(f"  Feasible: {feasible}")
    print()
    
    assert feasible, "Solution violates capacity constraints"
    
    return (selected, total_value)


if __name__ == "__main__":
    run()
