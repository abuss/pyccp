use pyccp_core::propagators::{LinearEq, LessThan, NeqLinear, Registry};
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

/// Test LessThan bounds propagation: x < y, x ∈ [5,10], y ∈ [5,10]
/// should tighten x.max to 9 and y.min to 6
#[test]
fn test_less_than_bounds_prune() {
    let mut s = Store::new(&[vec![5, 6, 7, 8, 9, 10], vec![5, 6, 7, 8, 9, 10]]);
    let mut r = Registry::new();
    r.add(Box::new(LessThan::new(0, 1)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // x should be pruned to [5, 9] (max tightened from 10 to 9)
    assert_eq!(s.domain(0).to_vec(), vec![5, 6, 7, 8, 9], "x should be [5, 9]");
    
    // y should be pruned to [6, 10] (min tightened from 5 to 6)
    assert_eq!(s.domain(1).to_vec(), vec![6, 7, 8, 9, 10], "y should be [6, 10]");
}

/// Test LessThan conflict detection: x < y but x ∈ [10,20], y ∈ [5,9]
/// should return Conflict because left.min (10) >= right.max (9)
#[test]
fn test_less_than_conflict() {
    let mut s = Store::new(&[vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20], 
                              vec![5, 6, 7, 8, 9]]);
    let mut r = Registry::new();
    r.add(Box::new(LessThan::new(0, 1)));
    
    // Should detect conflict from bounds reasoning
    assert_eq!(r.fixpoint(&mut s), Err(Conflict), "Should detect conflict from bounds");
}

/// Test NeqLinear bounds reasoning: x + y != 5 with x ∈ [2,3], y ∈ [2,2]
/// Only one open term (x) with y fixed to 2.
/// All-but-one logic: x can be 2 (sum=4, ok) or 3 (sum=5, violates !=)
/// So x should be pruned to [2]
#[test]
fn test_neq_linear_bounds_single_forcing() {
    let mut s = Store::new(&[vec![2, 3], vec![2]]);
    let mut r = Registry::new();
    r.add(Box::new(NeqLinear::new(vec![(0, 1), (1, 1)], 5)));
    
    // Fix y = 2
    s.commit(1, 2).unwrap();
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // x should be pruned to [2] (x=3 forces sum=5, violates !=)
    assert_eq!(s.domain(0).to_vec(), vec![2], "x should be pruned to [2]");
}

/// Test NeqLinear bounds reasoning: x + y != 20 with x ∈ [1,3], y ∈ [1,3]
/// k = 20 is outside possible range [min=2, max=6]
/// No pruning, no conflict (constraint is automatically satisfied)
#[test]
fn test_neq_linear_bounds_outside_range() {
    let mut s = Store::new(&[vec![1, 2, 3], vec![1, 2, 3]]);
    let mut r = Registry::new();
    r.add(Box::new(NeqLinear::new(vec![(0, 1), (1, 1)], 20)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // No pruning expected (constraint automatically satisfied)
    assert_eq!(s.domain(0).to_vec(), vec![1, 2, 3], "x domain unchanged");
    assert_eq!(s.domain(1).to_vec(), vec![1, 2, 3], "y domain unchanged");
}

/// Test NeqLinear bounds reasoning: x + y != 5 with both x, y open
/// x ∈ [2,3], y ∈ [2,3], k = 5
/// Bounds [min=4, max=6], k=5 is in range, but min < max (multiple solutions)
/// Bounds reasoning can't fix any value (all-but-one only prunes when exactly 1 open)
/// This tests that multiple open terms don't cause spurious pruning
#[test]
fn test_neq_linear_bounds_multiple_open_no_conflict() {
    let mut s = Store::new(&[vec![2, 3], vec![2, 3]]);
    let mut r = Registry::new();
    r.add(Box::new(NeqLinear::new(vec![(0, 1), (1, 1)], 5)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // No pruning expected (both can be 2 giving sum=4, both can be 3 giving sum=6,
    // or mixed giving sum=5; but sum=4 and sum=6 are achievable)
    assert_eq!(s.domain(0).to_vec(), vec![2, 3], "x domain unchanged");
    assert_eq!(s.domain(1).to_vec(), vec![2, 3], "y domain unchanged");
}

/// Test NeqLinear bounds reasoning: q1 - q2 != -1 (diagonal constraint for N-Queens)
/// Simulates column 0 and column 1 of 8-Queens
/// q1 ∈ [0,7], q2 ∈ [0,7], k = -1
/// All-but-one: if one is fixed, the other can avoid -1
/// Multiple open: bounds [min=-7, max=7], k=-1 in range, but no early conflict
#[test]
fn test_neq_linear_bounds_nqueens_diagonal() {
    let mut s = Store::new(&[vec![0, 1, 2, 3, 4, 5, 6, 7], 
                             vec![0, 1, 2, 3, 4, 5, 6, 7]]);
    let mut r = Registry::new();
    // q1 - q2 != -1 (anti-diagonal)
    r.add(Box::new(NeqLinear::new(vec![(0, 1), (1, -1)], -1)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // Multiple open terms: bounds [-7, 7], k=-1 in range
    // No bounds-based early conflict (need domain propagation or all-but-one)
    assert_eq!(s.domain(0).len(), 8, "q1 domain should not be pruned by bounds alone");
    assert_eq!(s.domain(1).len(), 8, "q2 domain should not be pruned by bounds alone");
}

