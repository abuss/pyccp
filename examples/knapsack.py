from pyccp import Store

def run():
    """Solve 0/1 knapsack: select items to maximize value under weight limit."""
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
    
    # Variables: selected[i] in {0, 1} — item i included or not
    s = Store([[0, 1] for _ in range(n)])
    
    # Enumerate all 2^n combinations, filter by weight, find max value
    sols = s.solve_all(limit=2**n)
    
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
