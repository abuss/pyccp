use pyccp_core::{Bounds, Conflict, Domain, Store};

#[test]
fn test_bounds_is_fixed() {
    let mut bounds_domain = Domain::Bounds(5, 5);
    assert!(bounds_domain.is_fixed());

    let mut range_domain = Domain::Bounds(1, 10);
    assert!(!range_domain.is_fixed());
}

#[test]
fn test_bounds_contains() {
    let bounds_domain = Domain::Bounds(1, 10);
    assert!(bounds_domain.contains(1));
    assert!(bounds_domain.contains(5));
    assert!(bounds_domain.contains(10));
    assert!(!bounds_domain.contains(0));
    assert!(!bounds_domain.contains(11));
    assert!(!bounds_domain.contains(-5));
}

#[test]
fn test_bounds_remove_boundary() {
    let mut bounds_domain = Domain::Bounds(1, 10);
    // Can remove lower boundary
    assert!(bounds_domain.remove(1));
    assert_eq!(bounds_domain, Domain::Bounds(2, 10));

    // Can remove upper boundary
    assert!(bounds_domain.remove(10));
    assert_eq!(bounds_domain, Domain::Bounds(2, 9));

    // Cannot remove interior values
    assert!(!bounds_domain.remove(5));
    assert_eq!(bounds_domain, Domain::Bounds(2, 9));
}

#[test]
fn test_store_hybrid_mixed_vec_and_bounds() {
    // Mix of small (Vec) and large (Bounds) domains
    let mut store = Store::new(&[
        vec![1, 2, 3],        // Small domain -> Domain::Vec
        (1..=1001).collect(), // Large domain -> Domain::Bounds (threshold 1000)
    ]);

    // Check that first domain is Vec
    assert!(store.is_vec_domain(0));
    assert!(!store.is_bounds_domain(0));
    assert_eq!(store.domain_or_none(0), Some(vec![1, 2, 3]));

    // Check that second domain is Bounds
    assert!(!store.is_vec_domain(1));
    assert!(store.is_bounds_domain(1));
    // Large domains don't expand to Vec
    assert_eq!(store.domain_or_none(1), None);
}

#[test]
fn test_store_bounds_trail_undo() {
    let mut store = Store::new(&[vec![], (1..=100).collect()]);
    
    // Mark before any operations
    let mark = store.mark();
    
    // Try to remove a boundary value from Bounds domain
    store
        .tighten_bounds(1, 10, 50)
        .expect("tighten_bounds should record trail");

    // Check bounds are tightened
    let b = store.bounds(1);
    assert_eq!(b.min, 10);
    assert_eq!(b.max, 50);

    // Undo should restore original bounds
    store.undo_to(mark);
    let b = store.bounds(1);
    assert_eq!(b.min, 1);
    assert_eq!(b.max, 100);
}

#[test]
fn test_store_threshold_converts_large_domains() {
    // Domain with exactly 1001 values should be Bounds
    let mut store = Store::new(&[(0..=1001).collect()]);
    assert!(store.is_bounds_domain(0));
    assert!(!store.is_vec_domain(0));

    // Domain with exactly 1000 values should be Vec
    let mut store = Store::new(&[(0..=999).collect()]);
    assert!(store.is_vec_domain(0));
    assert!(!store.is_bounds_domain(0));
}
