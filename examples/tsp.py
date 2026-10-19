from pyccp import Store

def run():
    """Solve TSP for 5 cities with in-solver cost and branch-and-bound optimization (v2.5).
    
    Uses Element constraint to compute edge costs directly in the solver:
    - tour[i] = destination city from city i (AllDiff permutation)
    - cost[i] = edge cost from i to tour[i] (Element lookup into dist[i])
    - total_cost = sum of cost[i] (LinearEq)
    - Objective: minimize total_cost (set_objective + B&B pruning)
    
    This enables branch-and-bound pruning during search: when a partial tour
    exceeds best_cost, the search prunes that branch automatically.
    """
    # Distance matrix: city i to city j
    dist = [
        [0, 2, 3, 4, 5],
        [2, 0, 1, 6, 3],
        [3, 1, 0, 2, 4],
        [4, 6, 2, 0, 1],
        [5, 3, 4, 1, 0],
    ]
    n = len(dist)
    
    # Variables layout:
    # - Cells 0..n-1: tour[i] = next city from city i
    # - Cells n..n*n+n-1: dist_cell[i][j] = distance from i to j
    # - Cells n+n*n..n+n*n+n-1: cost[i] = edge cost from i to tour[i]
    # - Cell n+n*n+n: total_cost = sum(cost[i])
    
    # Domains for tour: [0..n-1] permutation
    tour_domains = [list(range(n)) for _ in range(n)]
    
    # Domains for dist cells: fixed to distance matrix values
    dist_domains = [[dist[i][j]] for i in range(n) for j in range(n)]
    
    # Domains for cost: reasonable bounds from distance values
    all_dists = [dist[i][j] for i in range(n) for j in range(n)]
    cost_domains = [sorted(set(all_dists)) for _ in range(n)]
    
    # Domain for total_cost
    min_total = sum(min(dist[i]) for i in range(n))
    max_total = sum(max(dist[i]) for i in range(n))
    total_cost_domain = [list(range(min_total, max_total + 1))]
    
    all_domains = tour_domains + dist_domains + cost_domains + total_cost_domain
    s = Store(all_domains)
    
    # AllDiff: each city visited exactly once
    s.post(("all_diff", (list(range(n)),)))
    
    # No self-loops: tour[i] != i
    for i in range(n):
        s.post(("neq_linear", ([(i, 1)], i)))
    
    # Element constraints: cost[i] = dist[i][tour[i]]
    # dist_row_i starts at cell n + i*n
    for i in range(n):
        cost_cell = n + n * n + i
        tour_cell = i
        dist_row_start = n + i * n
        dist_row_cells = list(range(dist_row_start, dist_row_start + n))
        
        # Element: cost[i] = dist_row[i][tour[i]]
        s.post(("element", (dist_row_cells, tour_cell, cost_cell)))
    
    # LinearEq: total_cost = sum(cost[i])
    cost_terms = [(n + n * n + i, 1) for i in range(n)]
    total_cost_cell = n + n * n + n
    s.post(("linear_eq", (cost_terms + [(total_cost_cell, -1)], 0)))
    
    # Set objective: minimize total_cost
    s.set_objective(total_cost_cell, minimize=True)
    
    # Solve with branch-and-bound
    sols = s.solve_all(limit=1000)
    
    if not sols:
        print("tsp: no solution found")
        return None
    
    # Find best tour by cost (in v2.5, solver will prune automatically via set_objective())
    best_sol = None
    best_cost = float('inf')
    for sol in sols:
        # Extract tour from first n cells; remaining cells are dist, cost, total_cost
        tour = sol[:n]
        cost = sum(dist[i][tour[i]] for i in range(n))
        if cost < best_cost:
            best_cost = cost
            best_sol = tour
    
    assert best_sol is not None
    assert best_cost <= 12  # Known bound for this instance
    print(f"tsp: tour={best_sol}, cost={best_cost}")
    return (best_sol, best_cost)

if __name__ == "__main__":
    run()

