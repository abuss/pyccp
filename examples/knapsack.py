from pyccp import Store

def run():
    """Solve 0/1 knapsack with in-solver cost computation and branch-and-bound optimization.
    
    Uses LinearEq constraints to compute total value and weight.
    set_objective() enables B&B pruning to maximize value efficiently.
    """
    # Items: (value, weight)
    items = [
        (10, 5),   # item 0
        (8, 4),    # item 1
        (7, 3),    # item 2
        (6, 2),    # item 3
        (5, 1),    # item 4
        (4, 2),    # item 5
        (3, 1),    # item 6
        (2, 1),    # item 7
        (1, 1),    # item 8
        (1, 2),    # item 9
    ]
    n = len(items)
    capacity = 15
    
    # Variables: selected[0..n-1] ∈ {0,1}, value, weight
    # Cell indices: selected=0-9, value=10, weight=11
    s = Store(
        [[0, 1] for _ in range(n)] +       # selected[0..9], cells 0-9
        [list(range(50))] +                 # value, cell 10 (max possible: sum of all values)
        [list(range(50))]                  # weight, cell 11 (max possible: sum of all weights)
    )
    
    selected_start = 0
    value_cell = n
    weight_cell = n + 1
    
    # Constraint: Compute total value
    # value = sum(selected[i] * items[i].value for i in 0..n-1)
    # Expressed as: sum(selected[i] * items[i].value) - value = 0
    value_terms = [(i, items[i][0]) for i in range(n)] + [(value_cell, -1)]
    s.post(("linear_eq", (value_terms, 0)))
    
    # Constraint: Compute total weight
    # weight = sum(selected[i] * items[i].weight for i in 0..n-1)
    # Expressed as: sum(selected[i] * items[i].weight) - weight = 0
    weight_terms = [(i, items[i][1]) for i in range(n)] + [(weight_cell, -1)]
    s.post(("linear_eq", (weight_terms, 0)))
    
    # Constraint: Respect capacity limit
    # weight <= capacity
    s.post(("le_linear", ([(weight_cell, 1)], capacity)))
    
    # Optimization: Maximize value
    s.set_objective(value_cell, minimize=False)
    
    # Solve with branch-and-bound pruning
    # solve_all() explores all solutions with B&B pruning, tracking best_cost
    all_solutions = s.solve_all(limit=2**20)  # Limit prevents runaway; B&B prunes most branches
    
    if not all_solutions:
        print("knapsack: no solution found")
        return None
    
    # Find the solution with value equal to best_cost (optimal solution)
    best_cost = s.best_cost()
    if best_cost is None:
        print("knapsack: no solution found")
        return None
    
    # Best cost is the optimal value; find a solution with that value in value_cell
    solution = None
    for sol in all_solutions:
        if sol[value_cell] == best_cost:
            solution = sol
            break
    
    if not solution:
        # Fallback (shouldn't happen)
        solution = all_solutions[0]
    
    if not solution:
        print("knapsack: no solution found")
        return None
    
    # Extract selection and verify
    selected = solution[:n]
    total_value = sum(selected[i] * items[i][0] for i in range(n))
    total_weight = sum(selected[i] * items[i][1] for i in range(n))
    
    assert total_weight <= capacity
    assert total_value >= 25  # Known lower bound for this instance
    print(f"knapsack: selected={selected}, value={total_value}, weight={total_weight}")
    return (selected, total_value)
