use pyccp_core::{Bounds, Store};

#[test]
fn test_bounds_initialized_from_domain() {
    let store = Store::new(&[vec![1, 2, 3, 4, 5], vec![10, 20, 30]]);

    assert_eq!(store.bounds(0), Bounds::new(1, 5));
    assert_eq!(store.bounds(1), Bounds::new(10, 30));
}

#[test]
fn test_tighten_bounds_valid() {
    let mut store = Store::new(&[vec![1, 2, 3, 4, 5]]);
    store.tighten_bounds(0, Bounds::new(2, 4)).unwrap();
    assert_eq!(store.bounds(0), Bounds::new(2, 4));
}

#[test]
fn test_tighten_bounds_conflict() {
    let mut store = Store::new(&[vec![1, 2, 3, 4, 5]]);
    let result = store.tighten_bounds(0, Bounds::new(10, 20));
    assert!(result.is_err());
}

#[test]
fn test_bounds_contains() {
    let b = Bounds::new(5, 10);
    assert!(b.contains(7));
    assert!(!b.contains(3));
    assert!(!b.contains(11));
}
