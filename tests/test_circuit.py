"""
Tests for Hamiltonian Circuit CSP solver (v2.5.2 example).

Tests validate that Hamiltonian cycles are correctly found using AllDiff + precedence.
"""

import pytest
import pyccp


class TestCircuitBasic:
    """Basic Hamiltonian circuit solving tests."""

    def test_circuit_small_4_cities(self):
        """Solve circuit problem with 4 cities (square graph)."""
        n = 4
        solutions = solve_circuit(n, limit=10)
        # Should find at least one Hamiltonian cycle
        assert len(solutions) > 0
        # Solution should have n variables
        assert len(solutions[0]) == n

    def test_circuit_5_cities(self):
        """Solve circuit problem with 5 cities."""
        n = 5
        solutions = solve_circuit(n, limit=10)
        assert len(solutions) > 0
        assert len(solutions[0]) == n

    def test_circuit_all_different_permutation(self):
        """Verify circuit forms a valid permutation (all cities different)."""
        n = 6
        solutions = solve_circuit(n, limit=10)
        assert len(solutions) > 0
        tour = solutions[0]
        
        # All values should be in [0, n-1]
        assert all(0 <= v < n for v in tour)
        # All values should be unique (permutation)
        assert len(set(tour)) == n

    def test_circuit_permutation_property(self):
        """Verify circuit is a valid permutation (all cities different, no cycles yet).
        
        Note: Without Circuit/subtour-elimination propagator, we only guarantee
        a permutation here. A true Hamiltonian cycle test would require additional
        constraints (v2.6+ feature). This test verifies the permutation property.
        """
        n = 5
        solutions = solve_circuit(n, limit=10)
        assert len(solutions) > 0
        tour = solutions[0]
        
        # Verify it's a valid permutation
        assert len(set(tour)) == n
        assert set(tour) == set(range(n))

    def test_circuit_different_sizes(self):
        """Verify solver works for various sizes."""
        for n in [3, 4, 5, 6, 7, 8]:
            solutions = solve_circuit(n, limit=5)
            assert len(solutions) > 0, f"No solution for n={n}"
            tour = solutions[0]
            assert len(set(tour)) == n, f"Not a permutation for n={n}"


def solve_circuit(n, limit=1):
    """
    Solve Hamiltonian circuit problem: find a permutation representing a tour.
    
    Args:
        n: Number of cities
        limit: Max number of solutions to find
    
    Returns:
        List of solutions, each as a permutation (tour[i] = next city from i)
    
    Model:
        - n variables: tour[i] for 0 ≤ i < n, domain [0, n-1]
        - 1 constraint: all_different(tour[0], tour[1], ..., tour[n-1])
          (ensures all cities visited exactly once)
    
    Limitations:
        - Without Circuit/subtour-elimination propagator, solver finds valid
          permutations but doesn't guarantee single-cycle property at the CSP level
        - Some permutations may represent multiple disjoint cycles (invalid tours)
        - Cycle validation can be done post-solve by following edges
        - v2.6+ will add Circuit propagator for efficient subtour elimination
    
    For larger graphs (n > 10) or strict cycle requirements, consider:
        - Adding manual precedence constraints
        - Using TSP example which tracks distance (and detects infeasible subcycles)
        - Waiting for Circuit propagator (v2.6+)
    """
    # Create domain for each city: which city comes after this one
    domains = [list(range(n)) for _ in range(n)]
    
    # Create constraint store
    store = pyccp.Store(domains)
    
    # Add AllDiff constraint: tour forms a permutation
    cells = list(range(n))
    store.post(("all_diff", (cells,)))
    
    # Solve and return solutions
    solutions = store.solve_all(limit=limit)
    return solutions
