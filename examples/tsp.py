from pyccp import Store

def run():
    """Solve TSP for 5 cities with symmetric distance matrix."""
    # Distance matrix: city i to city j
    dist = [
        [0, 2, 3, 4, 5],
        [2, 0, 1, 6, 3],
        [3, 1, 0, 2, 4],
        [4, 6, 2, 0, 1],
        [5, 3, 4, 1, 0],
    ]
    n = len(dist)
    
    # Variables: tour[i] = next city after city i (permutation)
    # Domain: each tour[i] in 0..n-1
    s = Store([list(range(n)) for _ in range(n)])
    
    # AllDiff: each city visited exactly once
    s.post(("all_diff", (list(range(n)),)))
    
    # No self-loops: tour[i] != i for all i
    # Using neq_linear: (tour[i] - i) != 0
    for i in range(n):
        s.post(("neq_linear", ([(i, 1)], i)))
    
    # Enumerate all permutations
    sols = s.solve_all(limit=1000)
    
    if not sols:
        print("tsp: no solution found")
        return None
    
    # Compute cost for each solution and find the best
    best_sol = None
    best_cost = float('inf')
    for sol in sols:
        cost = sum(dist[i][sol[i]] for i in range(n))
        if cost < best_cost:
            best_cost = cost
            best_sol = sol
    
    assert best_sol is not None
    assert best_cost <= 12  # Known bound for this instance
    print(f"tsp: tour={best_sol}, cost={best_cost}")
    return (best_sol, best_cost)
