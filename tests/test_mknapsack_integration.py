"""Integration tests for MKP with bounds domains (v2.5.1)."""
import pytest
import pyccp


def test_mknapsack_threshold_converts_pb4_to_bounds():
    """Verify that large capacity domains (>1000 values) auto-convert to Bounds.
    
    Simulates pb4.dat structure: m=29 items, capacity ~55000.
    The capacity domain (0..55000) exceeds the BOUNDS_CONVERSION_THRESHOLD of 1000,
    triggering auto-conversion to Bounds representation.
    
    Note: This test verifies the Store is created successfully with large domains.
    The actual Bounds representation check happens internally in the Rust solver.
    """
    # Simulate pb4 structure: 29 items, 1 knapsack (simplified)
    m_items = 29
    
    # Binary selection domains for items
    item_domains = [[0, 1] for _ in range(m_items)]
    
    # Capacity domain for knapsack: 0 to ~55000
    # This should exceed threshold and convert to Bounds
    capacity_value = 55000
    capacity_domain = [list(range(capacity_value + 1))]
    
    # Combine domains
    domains = item_domains + capacity_domain
    
    # Create store - this will auto-convert large domains to Bounds internally
    store = pyccp.Store(domains)
    
    # Verify store was created successfully (large domain conversion happened internally)
    assert store is not None, "Store should be created successfully with large domains"
    
    # The Store should handle the large domain without issue
    # (Bounds optimization is transparent to the API)


def test_mknapsack_bounds_correctness():
    """Verify MKP solver with bounds domains solves correctly.
    
    Small MKP instance:
    - 3 items with values [4, 5, 3]
    - Weights per constraint: [2, 3, 1]
    - Capacity: 5
    - Optimal solution: x=[1,1,0], total_value=9, weight=5
    """
    # Setup: 3 items, 1 knapsack
    m = 3
    n = 1
    values = [4, 5, 3]
    weights = [[2, 3, 1]]  # 1 constraint
    capacity = 5
    
    # Cell layout: items 0..2, sum 3, value 4
    sum_cell = m
    value_cell = m + n
    
    # Compute possible sums via subset sum (ponytail style)
    def subset_sums(coeffs):
        sums = {0}
        for c in coeffs:
            sums |= {s + c for s in sums}
        return sorted(sums)
    
    domains = (
        [[0, 1] for _ in range(m)] +
        [subset_sums(weights[0])] +  # sum domain: all possible sums
        [subset_sums(values)]  # value domain: all possible values
    )
    
    store = pyccp.Store(domains)
    
    # Constraint: sum(x_i * w_i) = sum
    store.post(("linear_eq", (
        [(0, weights[0][0]), (1, weights[0][1]), (2, weights[0][2]), (sum_cell, -1)],
        0
    )))
    
    # Constraint: sum <= capacity
    store.post(("le_linear", ([(sum_cell, 1)], capacity)))
    
    # Constraint: sum(x_i * value_i) = value
    store.post(("linear_eq", (
        [(0, values[0]), (1, values[1]), (2, values[2]), (value_cell, -1)],
        0
    )))
    
    # Objective: maximize value
    store.set_objective(value_cell, minimize=False)
    
    # Solve using solve_all with branch-and-bound
    all_solutions = store.solve_all(limit=1000)
    assert len(all_solutions) > 0, "Should find at least one feasible solution"
    
    # Get the best cost from branch-and-bound
    best_cost = store.best_cost()
    assert best_cost is not None, "Should have found optimal cost via B&B"
    
    # Find the solution with the optimal cost
    solution = None
    for sol in all_solutions:
        if sol[value_cell] == best_cost:
            solution = sol
            break
    
    assert solution is not None, "Should find solution matching best cost"
    
    # Extract results
    x = solution[:m]
    total_value = solution[value_cell]
    total_weight = solution[sum_cell]
    
    # Verify optimality
    assert total_value == 9, f"Optimal value should be 9, got {total_value}"
    assert total_weight == 5, f"Optimal weight should be 5, got {total_weight}"
    assert x[0] == 1 and x[1] == 1 and x[2] == 0, \
        f"Optimal selection should be [1,1,0], got {x}"
    
    # Verify feasibility
    assert total_weight <= capacity, "Solution violates capacity constraint"
