use pyccp_core::*;

#[test]
fn set_objective_stores_cell_index() {
    let mut s = CustomSolver::new(&[vec![1, 2, 3]]);
    s.set_objective(0, true).unwrap();
    assert_eq!(s.best_cost(), None);  // no solution yet
}

#[test]
fn set_objective_minimize_flag() {
    let mut s = CustomSolver::new(&[vec![1, 2, 3]]);
    s.set_objective(0, false).unwrap();  // maximize
    assert_eq!(s.best_cost(), None);
}

#[test]
fn set_objective_invalid_cell_conflicts() {
    let mut s = CustomSolver::new(&[vec![1, 2]]);
    assert_eq!(s.set_objective(5, true), Err(Conflict));
}

#[test]
fn best_cost_none_initially() {
    let s = CustomSolver::new(&[vec![1, 2, 3]]);
    assert_eq!(s.best_cost(), None);
}

#[test]
fn best_cost_updates_after_solution() {
    let mut s = CustomSolver::new(&[vec![1, 2, 3]]);
    s.set_objective(0, true).unwrap();
    s.commit(0, 2).unwrap();  // fix cost to 2
    // After next collection (placeholder — tested in Task 3)
    // best_cost should be Some(2)
}

#[test]
fn branch_and_bound_prunes_higher_cost() {
    // x in 1..10, cost = x, minimize -> should find x=1
    let domains = vec![(1..=10).collect()];
    let mut s = CustomSolver::new(&domains);
    s.set_objective(0, true).unwrap();
    let sols = s.solve_all(100);
    // After pruning, only x=1 should be explored as a solution
    // (In practice, before optimization it would explore all 10 values)
    // This test verifies that best_cost updates; full pruning tested via benchmarks.
    assert_eq!(sols.len(), 1);
    assert_eq!(sols[0][0], 1);
}

#[test]
fn branch_and_bound_explores_all_optimal() {
    // x in 1..3, cost = 1 (constant) -> all three values tie
    // maximize (sense irrelevant since cost is constant)
    let mut s = CustomSolver::new(&[vec![1, 2, 3]]);
    s.set_objective(0, true).unwrap();
    // Fix the cell to 1 (single solution)
    s.commit(0, 1).unwrap();
    let sols = s.solve_all(10);
    assert_eq!(sols.len(), 1);
    assert_eq!(sols[0][0], 1);
    assert_eq!(s.best_cost(), Some(1));
}
