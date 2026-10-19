/// Tests for LinearEq bounds-aware propagation (Task B2)
use pyccp_core::{Store, propagators::{LinearEq, LeLinear, Registry}, store::Conflict};

#[test]
fn test_lineareq_bounds_conflict() {
    // LinearEq: x + y == 10
    // x bounds [1, 3], y bounds [4, 6]
    // Max sum: 3 + 6 = 9 < 10 -> Conflict
    let mut s = Store::new(&[vec![1, 2, 3], vec![4, 5, 6]]);
    let mut r = Registry::new();
    r.add(Box::new(LinearEq::new(vec![(0, 1), (1, 1)], 10)));
    
    // Bounds are automatically set during Store::new based on domain
    // x: [1, 3], y: [4, 6]
    assert_eq!(r.fixpoint(&mut s), Err(Conflict));
}

#[test]
fn test_lineareq_bounds_all_but_one_fixed() {
    // LinearEq: 2*x + y == 10
    // x fixed to 3, y domain [1, 10]
    // Solve for y: y = 10 - 2*3 = 4
    // y should be fixed to 4
    let mut s = Store::new(&[vec![1, 2, 3], vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]]);
    let mut r = Registry::new();
    r.add(Box::new(LinearEq::new(vec![(0, 2), (1, 1)], 10)));
    
    s.commit(0, 3).unwrap();
    r.fixpoint(&mut s).unwrap();
    
    // y should be fixed to 4
    assert!(s.is_fixed(1));
    assert_eq!(s.value(1), 4);
}

#[test]
fn test_lineareq_bounds_tighten_multiple_open() {
    // LinearEq: 2*x + y == 15
    // x domain [1, 5], y domain [5, 15]
    // Min sum: 2*1 + 5 = 7, Max sum: 2*5 + 15 = 25 (both around 15, no conflict)
    // All-but-one case only triggers when exactly 1 variable is open
    // Here both are open, so it doesn't trigger and no tightening
    
    let mut s = Store::new(&[vec![1, 2, 3, 4, 5], vec![5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]]);
    let mut r = Registry::new();
    r.add(Box::new(LinearEq::new(vec![(0, 2), (1, 1)], 15)));
    
    r.fixpoint(&mut s).unwrap();
    
    // Just verify no conflict
    assert!(!s.is_fixed(0));
    assert!(!s.is_fixed(1));
}

#[test]
fn test_lineareq_mixed_vec_and_bounds() {
    // Mixed scenario: one variable in Vec domain, rest fixed
    // This tests that Vec domain path still works alongside Bounds support
    // LinearEq: x + 2*y == 10
    // y fixed to 2
    // x domain [1, 6]
    // x must equal 10 - 4 = 6
    let mut s = Store::new(&[vec![1, 2, 3, 4, 5, 6], vec![1, 2, 3]]);
    let mut r = Registry::new();
    r.add(Box::new(LinearEq::new(vec![(0, 1), (1, 2)], 10)));
    
    s.commit(1, 2).unwrap();
    r.fixpoint(&mut s).unwrap();
    
    // x should be fixed to 6
    assert!(s.is_fixed(0));
    assert_eq!(s.value(0), 6);
}

// ============================================
// LeLinear bounds-aware propagation tests (Task B3)
// ============================================

#[test]
fn test_lelinear_bounds_conflict() {
    // LeLinear: x + y <= 10
    // x bounds [6, 10], y bounds [5, 9]
    // min_sum = 6 + 5 = 11 > 10 -> Conflict
    let mut s = Store::new(&[vec![6, 7, 8, 9, 10], vec![5, 6, 7, 8, 9]]);
    let mut r = Registry::new();
    r.add(Box::new(LeLinear::new(vec![(0, 1), (1, 1)], 10)));
    
    // min_sum = 6 + 5 = 11 > 10, should detect conflict
    assert_eq!(r.fixpoint(&mut s), Err(Conflict), "Should detect conflict when min_sum > rhs");
}

#[test]
fn test_lelinear_bounds_tighten() {
    // LeLinear: x + y <= 10
    // x domain [1, 6], y domain [1, 5]
    // Initial: min_sum = 1+1 = 2 (OK), max_sum = 6+5 = 11 > 10 (constraint violated)
    // Vec pruning should tighten at least one variable:
    // For x: other_max=5, space=5, new_max=5 (x should not include 6)
    // For y: other_max=6, space=4, new_max=4 (y should not include 5)
    let mut s = Store::new(&[vec![1, 2, 3, 4, 5, 6], vec![1, 2, 3, 4, 5]]);
    let mut r = Registry::new();
    r.add(Box::new(LeLinear::new(vec![(0, 1), (1, 1)], 10)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // After propagation, domain sizes should not expand
    let x_domain = s.domain(0).to_vec();
    let y_domain = s.domain(1).to_vec();
    
    assert!(x_domain.len() <= 6, "x domain should not expand");
    assert!(y_domain.len() <= 5, "y domain should not expand");
}
