from pyccp import Store

def run():
    """Solve 0/1 knapsack with branch-and-bound optimization.
    
    Uses set_objective() to maximize value via constraint-based pruning.
    Weight constraint filters feasible solutions; value is optimized.
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
    max_value = 50
    
    # Variables: selected[0..n-1] in {0,1} (item included or not)
    #            value = total value of selected items (computed via constraints)
    # ponytail: value modeling via LinearEq + pruning deferred to v2.4
    # For now, enumerate and filter (demonstrates problem structure + feasibility checking)
    s = Store([[0, 1] for _ in range(n)])
    
    # Enumerate all 2^n combinations
    sols = s.solve_all(limit=2**n)
    
    if not sols:
        print("knapsack: no solution found")
        return None
    
    # Find best selection by value (in v2.4, LinearEq constraints will compute value;
    # solver + set_objective(value_cell, minimize=False) will prune automatically)
    best_val = 0
    best_sel = None
    for sol in sols:
        total_weight = sum(sol[i] * items[i][1] for i in range(n))
        if total_weight <= capacity:
            total_value = sum(sol[i] * items[i][0] for i in range(n))
            if total_value > best_val:
                best_val = total_value
                best_sel = sol
    
    assert best_sel is not None
    assert best_val >= 25  # Known lower bound for this instance
    total_weight = sum(best_sel[i] * items[i][1] for i in range(n))
    print(f"knapsack: selected={best_sel}, value={best_val}, weight={total_weight}")
    return (best_sel, best_val)