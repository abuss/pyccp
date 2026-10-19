use pyccp_core::store::Store;
use pyccp_core::propagators::{AllDiff, NeqLinear, Registry};

#[test]
fn test_lazy_queue_skips_unaffected() {
    // Post two independent allDiff constraints
    // Change only one; should not affect the other
    let mut store = Store::new(&[
        vec![1, 2],
        vec![1, 2],
        vec![1, 2],
        vec![1, 2],
    ]);
    
    let mut registry = Registry::new();
    
    // First constraint: AllDiff on variables [0, 1]
    registry.add(Box::new(AllDiff::new(vec![0, 1])));
    registry.fixpoint(&mut store).unwrap();
    
    // Second constraint: AllDiff on variables [2, 3]
    registry.add(Box::new(AllDiff::new(vec![2, 3])));
    registry.fixpoint(&mut store).unwrap();
    
    // Now fix variable 0 to 1
    store.commit(0, 1).unwrap();
    registry.fixpoint(&mut store).unwrap();
    
    // Variable 1 should be pruned (from constraint 0)
    let v1_vals: std::vec::Vec<i64> = store.domain(1).iter().copied().collect();
    assert_eq!(v1_vals, vec![2], "variable 1 should be pruned to [2]");
    
    // Variables 2 and 3 should be UNCHANGED (constraint 1 was never triggered for them)
    let v2_vals: std::vec::Vec<i64> = store.domain(2).iter().copied().collect();
    let v3_vals: std::vec::Vec<i64> = store.domain(3).iter().copied().collect();
    assert_eq!(v2_vals, vec![1, 2], "variable 2 should still be [1, 2]");
    assert_eq!(v3_vals, vec![1, 2], "variable 3 should still be [1, 2]");
}

#[test]
fn test_lazy_queue_cascading() {
    // Post AllDiff constraint for [x, y, z]
    // When we fix x=1, the constraint should prune 1 from y and z
    let mut store = Store::new(&[
        vec![1, 2, 3],
        vec![1, 2, 3],
        vec![1, 2, 3],
    ]);
    
    let mut registry = Registry::new();
    registry.add(Box::new(AllDiff::new(vec![0, 1, 2])));
    registry.fixpoint(&mut store).unwrap();
    
    // Fix variable 0 to 1
    store.commit(0, 1).unwrap();
    registry.fixpoint(&mut store).unwrap();
    
    // Variables 1 and 2 should have 1 removed
    let v1_vals: std::vec::Vec<i64> = store.domain(1).iter().copied().collect();
    let v2_vals: std::vec::Vec<i64> = store.domain(2).iter().copied().collect();
    assert_eq!(v1_vals, vec![2, 3], "variable 1 should have 1 removed");
    assert_eq!(v2_vals, vec![2, 3], "variable 2 should have 1 removed");
}

#[test]
fn test_lazy_queue_multi_constraint_interaction() {
    // Test that multiple constraints properly cascade through the lazy queue
    // When we fix a variable, it should trigger all propagators that touch it,
    // which may in turn trigger other propagators
    let mut store = Store::new(&[
        vec![1, 2, 3],
        vec![1, 2, 3],
        vec![1, 2, 3],
    ]);
    
    let mut registry = Registry::new();
    
    // Add constraint: x - y != 0
    registry.add(Box::new(NeqLinear::new(vec![(0, 1), (1, -1)], 0)));
    // Add constraint: y - z != 0
    registry.add(Box::new(NeqLinear::new(vec![(1, 1), (2, -1)], 0)));
    
    registry.fixpoint(&mut store).unwrap();
    
    // Fix x to 1
    store.commit(0, 1).unwrap();
    registry.fixpoint(&mut store).unwrap();
    
    // The first constraint (x - y != 0) should prune 1 from y
    let v1_vals: std::vec::Vec<i64> = store.domain(1).iter().copied().collect();
    assert_eq!(v1_vals, vec![2, 3], "variable 1 should have 1 removed");
}
