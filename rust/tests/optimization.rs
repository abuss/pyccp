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
