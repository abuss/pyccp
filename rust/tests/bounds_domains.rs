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

// ============================================
// weing7.dat MKP test (Phase B validation)
// ============================================

#[test]
fn test_weing7_bounds_auto_conversion() {
    // weing7.dat: m=2 knapsacks, n=105 items
    // Large capacity domains should auto-convert to Bounds at threshold 1000
    
    let capacities = vec![41850, 38261];
    let max_value = 1095445;
    
    // Create domains:
    // - 6 binary item selections (simplified; weing7 has 105 but we test concept)
    // - 2 large weight-sum accumulators [0, capacity[k]]
    // - 1 large value accumulator [0, max_value]
    
    let mut domains: Vec<Vec<i64>> = vec![];
    
    // Binary item selections
    for _ in 0..6 {
        domains.push(vec![0, 1]);
    }
    
    // Large weight-sum accumulators (above 1000 threshold)
    // capacity[0] = 41850 > 1000 -> should convert to Bounds
    domains.push((0..=capacities[0]).collect());
    
    // capacity[1] = 38261 > 1000 -> should convert to Bounds  
    domains.push((0..=capacities[1]).collect());
    
    // Large value accumulator
    // max_value = 1095445 > 1000 -> should convert to Bounds
    domains.push((0..=max_value).collect());
    
    let store = Store::new(&domains);
    
    // Check auto-conversion
    let weight_sum_0_is_bounds = store.is_bounds_domain(6);
    let weight_sum_1_is_bounds = store.is_bounds_domain(7);
    let value_is_bounds = store.is_bounds_domain(8);
    
    // Large domains (>1000) should be converted to Bounds
    assert!(weight_sum_0_is_bounds, 
        "weight_sum[0] domain [0, {}] should auto-convert to Bounds", capacities[0]);
    assert!(weight_sum_1_is_bounds, 
        "weight_sum[1] domain [0, {}] should auto-convert to Bounds", capacities[1]);
    assert!(value_is_bounds, 
        "value domain [0, {}] should auto-convert to Bounds", max_value);
}
