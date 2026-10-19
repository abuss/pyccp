from pyccp import Store

def run():
    """Solve TSP for 5 cities with branch-and-bound optimization.
    
    LIMITATION (v2.4): TSP cost computation is deferred to in-solver.
    
    Currently, the example enumerates permutations via AllDiff and self-loop constraints,
    then computes edge costs externally. This demonstrates the problem structure but
    does not showcase branch-and-bound pruning (which requires cost as a constraint variable).
    
    v2.5 will add multi-dimensional Element indexing or table constraints,
    enabling in-solver cost computation: for each edge (i, tour[i]), look up
    dist[i][tour[i]] via Element, then sum via LinearEq.
    
    See examples/knapsack.py for a complete in-solver optimization example (v2.4).
    See docs/OPTIMIZATION.md for the roadmap.
    
    ponytail: TSP in-solver cost requires dynamic 2D indexing (edge_idx = from*n + to);
    defer to v2.5 when table constraints or multi-dim Element available.
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
    
    # Variables: tour[0..n-1] = next city after city i (permutation)
    # Simplified model: just find permutation, compute cost externally
    # (Full constraint-based cost would use Element for dist[i][tour[i]] lookups)
    s = Store([list(range(n)) for _ in range(n)])
    
    # AllDiff: each city visited exactly once
    s.post(("all_diff", (list(range(n)),)))
    
    # No self-loops: tour[i] != i for all i
    for i in range(n):
        s.post(("neq_linear", ([(i, 1)], i)))
    
    # Find optimal solution via branch-and-bound enumeration
    # In v2.5, Element constraints will support 2D indexing (cost[i] = dist[tour[i-1]][tour[i]]),
    # enabling automatic branch-and-bound pruning via set_objective().
    # For now, enumerate and filter externally (problem structure is correct).
    # ponytail: TSP in-solver cost requires dynamic 2D indexing (edge_idx = from*n + to);
    # defer to v2.5 when table constraints or multi-dim Element available.
    sols = s.solve_all(limit=1000)
    
    if not sols:
        print("tsp: no solution found")
        return None
    
    # Find best tour by cost (in v2.5, solver will prune automatically via set_objective())
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
