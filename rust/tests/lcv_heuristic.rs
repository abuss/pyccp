/// Test suite for LCV (Least Constraining Value) heuristic
/// LCV orders values by how constraining they are; less constraining values are tried first.

#[cfg(test)]
mod lcv_heuristic_tests {
    use pyccp_core::solver::{CustomSolver, Constraint, Solver};

    #[test]
    fn lcv_orders_values_by_propagator_count() {
        // Test that value_order returns values in propagator-count order
        // Cells with more propagators = more constraining = tried later
        let domains: Vec<Vec<i64>> = vec![vec![1, 2, 3], vec![1, 2, 3]];
        let s = CustomSolver::new(&domains);
        
        // Initially, cell 0 has no propagators, so value_score should return 0
        // This is a sanity check that the method exists and runs
        let order = s.value_order(0);
        assert_eq!(order.len(), 3);
        assert_eq!(order, vec![1, 2, 3]); // No reordering when all tied at 0 propagators
    }

    #[test]
    fn lcv_solves_eight_queens() {
        // Test that LCV heuristic still solves N-Queens correctly
        // (regression test: LCV should not break correctness)
        let domains: Vec<Vec<i64>> = (0..8).map(|_| (0..8).collect()).collect();
        let mut s = CustomSolver::new(&domains);
        
        for i in 0..8 {
            for j in (i + 1)..8 {
                s.post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], 0)).unwrap();
                s.post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], i as i64 - j as i64)).unwrap();
                s.post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], j as i64 - i as i64)).unwrap();
            }
        }
        
        let sol = s.solve();
        assert!(sol.is_some(), "8-Queens should have a solution with LCV heuristic");
        
        let sol = sol.unwrap();
        assert_eq!(sol.len(), 8);
        
        // Verify solution is valid
        for row in 0..8 {
            for col in (row + 1)..8 {
                assert_ne!(sol[row], sol[col], "column conflict");
                assert_ne!(sol[row] - row as i64, sol[col] - col as i64, "diagonal conflict");
                assert_ne!(sol[row] + row as i64, sol[col] + col as i64, "anti-diagonal conflict");
            }
        }
    }
}
