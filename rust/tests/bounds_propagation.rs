use pyccp_core::propagators::{LinearEq, LessThan, NeqLinear, Multiply, Element, AllDiff, Registry};
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

/// Test Multiply bounds divisibility: x * y = 12 with x ∈ [3,5], y ∈ [1,10]
/// Divisibility: both x and y must divide 12
/// Valid divisors of 12: ±1, ±2, ±3, ±4, ±6, ±12
/// x ∈ [3,5] ∩ divisors = {3, 4}, y ∈ [1,10] ∩ divisors = {1,2,3,4,6}
#[test]
fn test_multiply_bounds_divisibility_pruning() {
    let mut s = Store::new(&[vec![3, 4, 5], vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]]);
    let mut r = Registry::new();
    r.add(Box::new(Multiply::new(0, 1, 12)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // x should be pruned to divisors of 12 in [3,5]
    assert_eq!(s.domain(0).to_vec(), vec![3, 4], "x should be pruned to divisors of 12");
    
    // y should be pruned to divisors of 12 in [1,10]
    assert_eq!(s.domain(1).to_vec(), vec![1, 2, 3, 4, 6], "y should be pruned to divisors of 12");
}

/// Test Multiply with one side fixed: x * y = 12, fix x = 3 -> y = 4
#[test]
fn test_multiply_bounds_one_side_fixed() {
    let mut s = Store::new(&[vec![3], vec![1, 2, 3, 4, 5, 6, 10, 12]]);
    let mut r = Registry::new();
    r.add(Box::new(Multiply::new(0, 1, 12)));
    
    // Fix x = 3
    s.commit(0, 3).unwrap();
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // y should be fixed to 4 (12 / 3 = 4)
    assert!(s.is_fixed(1), "y should be fixed");
    assert_eq!(s.value(1), 4, "y should be 4");
}

/// Test Multiply bounds divisibility: x * y = 12 with both open
/// x ∈ [3,5], y ∈ [1,10]
/// Divisors of 12 in x: {3,4}, divisors of 12 in y: {1,2,3,4,6}
/// Bounds reasoning should prune non-divisors from both domains
#[test]
fn test_multiply_bounds_prune_invalid_divisors() {
    let mut s = Store::new(&[vec![3, 4, 5], vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]]);
    let mut r = Registry::new();
    r.add(Box::new(Multiply::new(0, 1, 12)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // x: 5 is not a divisor of 12, should be pruned
    assert!(!s.domain(0).to_vec().contains(&5), "5 is not a divisor of 12, should be pruned from x");
    assert_eq!(s.domain(0).to_vec(), vec![3, 4], "x should contain only divisors of 12");
    
    // y: 5,7,8,9,10 are not divisors of 12, should be pruned
    let y_vals = s.domain(1).to_vec();
    assert!(!y_vals.contains(&5), "5 is not a divisor of 12");
    assert!(!y_vals.contains(&7), "7 is not a divisor of 12");
    assert!(!y_vals.contains(&8), "8 is not a divisor of 12");
    assert_eq!(s.domain(1).to_vec(), vec![1, 2, 3, 4, 6], "y should contain only divisors of 12");
}

/// Test Multiply with zero: x * y = 0, x ∈ [0,5], y ∈ [0,10]
/// If one is 0, the other can be anything; if neither 0, product != 0
#[test]
fn test_multiply_bounds_zero_constraint() {
    let mut s = Store::new(&[vec![0, 1, 2, 3, 4, 5], vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]]);
    let mut r = Registry::new();
    r.add(Box::new(Multiply::new(0, 1, 0)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // Both have 0 in domain initially (at least one must be 0)
    // Bounds reasoning: both must divide 0 (only 0 divides 0 conceptually, but we handle it specially)
    assert!(s.domain(0).contains(0), "x should contain 0");
    assert!(s.domain(1).contains(0), "y should contain 0");
}

/// Test Element bounds filtering: y = array[idx]
/// array = [5, 15, 25], idx ∈ [0,2], y ∈ [10,20]
/// array[0] = 5 ∉ [10,20] → prune idx = 0
/// array[1] = 15 ∈ [10,20] → keep idx = 1
/// array[2] = 25 ∉ [10,20] → prune idx = 2
/// Result: idx = 1
#[test]
fn test_element_bounds_prunes_idx() {
    // Fixed array: cells [6,7,8] represent array elements (fixed to 5,15,25)
    // idx = 9 (can be 0,1,2)
    // y = 10 (can be 10-20)
    let mut s = Store::new(&[
        vec![5],         // array[0] fixed to 5
        vec![15],        // array[1] fixed to 15
        vec![25],        // array[2] fixed to 25
        vec![0, 1, 2],   // idx can be 0, 1, or 2
        vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20], // y ∈ [10,20]
    ]);
    let mut r = Registry::new();
    r.add(Box::new(Element::new(vec![0, 1, 2], 3, 4)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // idx should be pruned to [1] (only array[1]=15 intersects with y ∈ [10,20])
    assert_eq!(s.domain(3).to_vec(), vec![1], "idx should be pruned to [1]");
}

/// Test Element bounds filtering: y = array[idx]
/// array cells ∈ [1,10], idx ∈ [0,2], y ∈ [5,8]
/// All indices have potential values in [1,10], y needs [5,8]
/// Index filtering keeps indices where array value could be in [5,8]
/// Note: Current Element impl only prunes idx/y when one is fixed;
/// bounds-based bidirectional pruning is future work
#[test]
fn test_element_bounds_multiple_valid_indices() {
    let mut s = Store::new(&[
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],   // array[0]
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],   // array[1]
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],   // array[2]
        vec![0, 1, 2],                          // idx
        vec![5, 6, 7, 8],                       // y ∈ [5,8]
    ]);
    let mut r = Registry::new();
    r.add(Box::new(Element::new(vec![0, 1, 2], 3, 4)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // All indices are valid (all array cells have bounds [1,10] which intersect [5,8])
    assert_eq!(s.domain(3).to_vec(), vec![0, 1, 2], "All indices should be valid");
    
    // array cells are not pruned when both idx and y are open (not yet implemented)
    // This is where v2.3 bidirectional pruning would apply
    for arr_idx in 0..3 {
        let arr_len = s.domain(arr_idx).len();
        assert_eq!(arr_len, 10, "array[{}] not pruned (both idx and y open)", arr_idx);
    }
}

/// Test Element bounds filtering: y = array[idx]
/// array cells ∈ [1,3], idx ∈ [0,2], y ∈ [10,20]
/// No array value can be in [10,20], so no valid index
#[test]
fn test_element_bounds_no_intersection() {
    let mut s = Store::new(&[
        vec![1, 2, 3],     // array[0]
        vec![1, 2, 3],     // array[1]
        vec![1, 2, 3],     // array[2]
        vec![0, 1, 2],     // idx
        vec![10, 11, 12], // y ∈ [10,12], no intersection with [1,3]
    ]);
    let mut r = Registry::new();
    r.add(Box::new(Element::new(vec![0, 1, 2], 3, 4)));
    
    // Run propagation to fixpoint
    // No valid indices (no array value intersects with y bounds)
    assert_eq!(r.fixpoint(&mut s), Err(Conflict), "Should detect conflict: no valid indices");
}

/// Test AllDiff bounds reasoning: disjoint intervals pigeonhole principle.
/// Variables with bounds that don't cause conflicts:
///   x0 ∈ [1, 2]
///   x1 ∈ [3, 4]
///   x2 ∈ [5, 6]
#[test]
fn test_alldiff_bounds_disjoint_intervals() {
    let mut s = Store::new(&[
        vec![1, 2],     // x0 ∈ [1, 2]
        vec![3, 4],     // x1 ∈ [3, 4]
        vec![5, 6],     // x2 ∈ [5, 6]
    ]);
    let mut r = Registry::new();
    r.add(Box::new(AllDiff::new(vec![0, 1, 2])));
    
    let result = r.fixpoint(&mut s);
    assert!(result.is_ok(), "Disjoint intervals should not cause conflict");
}

/// Test AllDiff bounds pigeonhole conflict: too many vars in too small an interval.
#[test]
fn test_alldiff_bounds_pigeonhole_conflict() {
    let mut s = Store::new(&[
        vec![1, 2],      // x0 ∈ [1, 2]
        vec![1, 2],      // x1 ∈ [1, 2]
        vec![1, 2],      // x2 ∈ [1, 2]
    ]);
    let mut r = Registry::new();
    r.add(Box::new(AllDiff::new(vec![0, 1, 2])));
    
    assert_eq!(r.fixpoint(&mut s), Err(Conflict), "Should detect pigeonhole conflict: 3 vars in [1, 2]");
}

/// Test AllDiff bounds exact fit: vars tighten to constrained interval.
#[test]
fn test_alldiff_bounds_exact_fit() {
    let mut s = Store::new(&[
        vec![1, 2],       // x0: bounds=[1,2], domain=[1,2]
        vec![3, 4],       // x1: bounds=[3,4], domain=[3,4]
        vec![1, 2, 3, 4], // x2: bounds=[1,4], domain=[1,2,3,4]
    ]);
    let mut r = Registry::new();
    r.add(Box::new(AllDiff::new(vec![0, 1, 2])));
    
    r.fixpoint(&mut s).unwrap();
    
    let x2_bounds = s.bounds(2);
    assert_eq!(x2_bounds.min, 1, "x2 bounds.min should be tightened to 1");
    assert_eq!(x2_bounds.max, 2, "x2 bounds.max should be tightened to 2");
}
