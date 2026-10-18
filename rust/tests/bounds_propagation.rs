use pyccp_core::propagators::{LinearEq, Registry};
use pyccp_core::store::{Store, Conflict};

/// Test bounds conflict detection: x + y = 100, x ∈ [0,5], y ∈ [0,5]
/// Sum max = 5 + 5 = 10, but constraint requires 100 -> conflict
#[test]
fn test_linear_eq_bounds_conflict() {
    let mut s = Store::new(&[vec![0, 1, 2, 3, 4, 5], vec![0, 1, 2, 3, 4, 5]]);
    let mut r = Registry::new();
    r.add(Box::new(LinearEq::new(vec![(0, 1), (1, 1)], 100)));
    
    // Should detect conflict from bounds reasoning (max sum 10 < rhs 100)
    assert_eq!(r.fixpoint(&mut s), Err(Conflict), "Should detect conflict from bounds");
}

/// Test that unfixed variables prune with propagation iteration.
/// x + y = 10, x ∈ [0,5], y ∈ [0,5] initially has no solution,
/// but if one becomes fixed by AllDiff or similar, the other gets constrained.
#[test]
fn test_linear_eq_with_fixed_variable() {
    // x + y = 10, fix x = 5 -> y must be 5
    let mut s = Store::new(&[vec![0, 1, 2, 3, 4, 5], vec![0, 1, 2, 3, 4, 5]]);
    let mut r = Registry::new();
    r.add(Box::new(LinearEq::new(vec![(0, 1), (1, 1)], 10)));
    
    // Manually fix x = 5
    s.commit(0, 5).unwrap();
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // y should be fixed to 5
    assert!(s.is_fixed(1), "y should be fixed");
    assert_eq!(s.value(1), 5, "y should be 5");
}

