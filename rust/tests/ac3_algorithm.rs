use pyccp_core::propagators::{AllDiff, Registry};
use pyccp_core::store::{Store, Conflict};

/// Test AC-3 Phase 1: Removing fixed values from other variables
/// allDiff([x, y, z]), x=1 should remove 1 from y and z domains
#[test]
fn test_ac3_removes_fixed_from_others() {
    // Use large domains to force SimpleIntersection algorithm (need average > 500)
    let mut s = Store::new(&[
        vec![1],
        (1..=1001).collect::<Vec<i64>>(),
        (1..=1001).collect::<Vec<i64>>(),
    ]);
    
    let mut r = Registry::new();
    r.add(Box::new(AllDiff::new(vec![0, 1, 2], &s)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // x should remain 1
    assert_eq!(s.value(0), 1);
    
    // 1 should be removed from y's domain
    assert!(!s.domain(1).contains(1), "1 should be removed from y");
    assert!(s.domain(1).contains(2), "2 should remain in y");
    assert!(s.domain(1).contains(3), "3 should remain in y");
    
    // 1 should be removed from z's domain
    assert!(!s.domain(2).contains(1), "1 should be removed from z");
    assert!(s.domain(2).contains(2), "2 should remain in z");
    assert!(s.domain(2).contains(3), "3 should remain in z");
}

/// Test AC-3 Phase 2: Fixing unique supporters
/// allDiff([x, y, z]), x∈{1,2}, y∈{2,3}, z∈{3,4}
/// - Value 1 only in x => x must be 1
/// - After x=1: value 2 only in y => y must be 2
/// - After y=2: value 3 only in z => z must be 3
#[test]
fn test_ac3_fixes_unique_supporters() {
    // Use large domains to force SimpleIntersection algorithm
    let mut s = Store::new(&[
        (1..=501).collect::<Vec<i64>>(),
        (2..=502).collect::<Vec<i64>>(),
        (3..=503).collect::<Vec<i64>>(),
    ]);
    
    let mut r = Registry::new();
    r.add(Box::new(AllDiff::new(vec![0, 1, 2], &s)));
    
    // Run propagation to fixpoint
    r.fixpoint(&mut s).unwrap();
    
    // All variables should be fixed through cascading unique supporter fixing
    assert!(s.is_fixed(0), "x should be fixed");
    assert_eq!(s.value(0), 1, "x should be 1");
    
    assert!(s.is_fixed(1), "y should be fixed");
    assert_eq!(s.value(1), 2, "y should be 2");
    
    assert!(s.is_fixed(2), "z should be fixed");
    assert_eq!(s.value(2), 3, "z should be 3");
}

/// Test AC-3 conflict detection: Unsupported values
/// allDiff([x, y]), x∈{1}, y∈{1}
/// Both variables must be assigned value 1, which violates AllDiff -> Conflict
#[test]
fn test_ac3_conflict_no_support() {
    let mut s = Store::new(&[
        vec![1],
        vec![1],
    ]);
    
    let mut r = Registry::new();
    r.add(Box::new(AllDiff::new(vec![0, 1], &s)));
    
    // Should detect conflict during fixpoint
    let result = r.fixpoint(&mut s);
    assert_eq!(result, Err(Conflict), "Should detect conflict when AllDiff can't be satisfied");
}
