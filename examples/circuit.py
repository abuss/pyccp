"""
Hamiltonian Circuit CSP Solver (v2.5.2)

Finds valid permutations representing tours through n cities using AllDiff constraints.
Note: Without Circuit propagator (v2.6+), solver finds permutations but not guaranteed
single-cycle property. For full Hamiltonian cycle verification, see TSP example.
"""

from pyccp import Store


def solve_circuit(n, limit=1):
    """
    Find valid permutations representing tours through n cities.
    
    Args:
        n: Number of cities
        limit: Max number of solutions to find (default: 1)
    
    Returns:
        List of solutions, each as a permutation (tour[i] = next city from i)
    """
    # Create domain for each city: which city comes after this one
    domains = [list(range(n)) for _ in range(n)]
    
    # Create constraint store
    store = Store(domains)
    
    # Add AllDiff constraint: all cities appear exactly once in tour
    cells = list(range(n))
    store.post(("all_diff", (cells,)))
    
    # Solve and return solutions
    solutions = store.solve_all(limit=limit)
    return solutions


def validate_hamiltonian_cycle(tour):
    """
    Validate that a permutation forms a single Hamiltonian cycle (no subcycles).
    
    Args:
        tour: Permutation where tour[i] = next city from city i
    
    Returns:
        True if tour is a valid Hamiltonian cycle, False if it contains subcycles
    """
    n = len(tour)
    visited = set()
    current = 0
    
    for _ in range(n):
        if current in visited:
            return False  # Found a cycle before visiting all cities
        visited.add(current)
        current = tour[current]
    
    # After n steps, should be back at start and have visited all cities
    return current == 0 and len(visited) == n


def find_hamiltonian_cycles(n, limit=1):
    """
    Find only valid Hamiltonian cycles (no subcycles).
    
    This is a convenience wrapper that:
    1. Finds permutations using solve_circuit()
    2. Filters to keep only valid single-cycle tours
    
    Args:
        n: Number of cities
        limit: Approx number of valid cycles to find (may find fewer)
    
    Returns:
        List of valid Hamiltonian cycles (guaranteed single-cycle)
    """
    # To get ~limit valid cycles, search for more permutations
    search_limit = limit * (n // 2)
    
    permutations = solve_circuit(n, limit=search_limit)
    valid_cycles = [perm for perm in permutations if validate_hamiltonian_cycle(perm)]
    
    return valid_cycles[:limit]


def run():
    """Example: find Hamiltonian circuits."""
    n = 5
    print(f"Finding Hamiltonian circuit permutations for {n} cities...")
    print()
    
    # Find permutations (may include subcycles)
    permutations = solve_circuit(n, limit=10)
    print(f"Found {len(permutations)} permutation(s)")
    for i, perm in enumerate(permutations):
        is_cycle = validate_hamiltonian_cycle(perm)
        status = "✅ Valid cycle" if is_cycle else "⚠️  Contains subcycles"
        print(f"  {i+1}. {perm} — {status}")
    
    print()
    print(f"Filtering to valid Hamiltonian cycles...")
    valid_cycles = find_hamiltonian_cycles(n, limit=5)
    print(f"Found {len(valid_cycles)} valid cycle(s)")
    for i, cycle in enumerate(valid_cycles):
        print(f"  {i+1}. {cycle}")
    
    print()
    print("Note: For full Hamiltonian cycle solving with distance optimization,")
    print("see the TSP (Travelling Salesman Problem) example.")
    
    return valid_cycles


if __name__ == "__main__":
    run()
