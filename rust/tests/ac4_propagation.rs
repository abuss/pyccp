use pyccp_core::store::Store;
use pyccp_core::propagators::{should_use_ac4, AllDiffState, AllDiffAlgorithm, propagate_ac4, AllDiff, Propagator};

#[test]
fn test_ac4_enabled_for_small_domains() {
    // Create store with small domains: [1,2,3] and [1,2,3]
    // Call should_use_ac4 with vars [0, 1]
    // Assert: should_use_ac4 returns true
    let store = Store::new(&[vec![1, 2, 3], vec![1, 2, 3]]);
    let vars = vec![0, 1];
    assert!(should_use_ac4(&vars, &store));
}

#[test]
fn test_ac4_disabled_for_large_domains() {
    // Create store with large domains: [0..1001] and [0..1001]
    // Call should_use_ac4 with vars [0, 1]
    // Assert: should_use_ac4 returns false (exceeds threshold)
    let domain_0: Vec<i64> = (0..=1000).collect();
    let domain_1: Vec<i64> = (0..=1000).collect();
    let store = Store::new(&[domain_0, domain_1]);
    let vars = vec![0, 1];
    assert!(!should_use_ac4(&vars, &store));
}

#[test]
fn test_ac4_disabled_for_bounds_domains() {
    // Create store with Bounds domain (auto-converts at ~1100 values)
    // Call should_use_ac4 with those vars
    // Assert: should_use_ac4 returns false (Bounds domain detected)
    let domain_0: Vec<i64> = (0..=1100).collect();
    let domain_1: Vec<i64> = (1..=101).collect();
    let store = Store::new(&[domain_0, domain_1]);
    let vars = vec![0, 1];
    assert!(!should_use_ac4(&vars, &store));
}

#[test]
fn test_support_map_construction() {
    // Setup: 3 variables with domains [1,2], [1,3], [2,3]
    let store = Store::new(&[vec![1, 2], vec![1, 3], vec![2, 3]]);
    let vars = vec![0, 1, 2];
    
    // Call: compute_support_map
    let support_map = pyccp_core::propagators::compute_support_map(&vars, &store);
    
    // Assert: support_map.get(&(0, 1)) has entries for (1,3), (2,2), (2,3)
    let supports_0_1 = support_map.get(&(0, 1)).expect("support_map should have entry for (0, 1)");
    assert!(supports_0_1.contains(&(1, 3)), "support_map[(0, 1)] should contain (1, 3)");
    assert!(supports_0_1.contains(&(2, 2)), "support_map[(0, 1)] should contain (2, 2)");
    assert!(supports_0_1.contains(&(2, 3)), "support_map[(0, 1)] should contain (2, 3)");
    
    // Assert: support_map.get(&(0, 2)) has entries for (1,1), (1,3), (2,3)
    let supports_0_2 = support_map.get(&(0, 2)).expect("support_map should have entry for (0, 2)");
    assert!(supports_0_2.contains(&(1, 1)), "support_map[(0, 2)] should contain (1, 1)");
    assert!(supports_0_2.contains(&(1, 3)), "support_map[(0, 2)] should contain (1, 3)");
    assert!(supports_0_2.contains(&(2, 3)), "support_map[(0, 2)] should contain (2, 3)");
    
    // Assert: support_map contains exactly 6 entries total (2+2+2 values)
    assert_eq!(support_map.len(), 6, "support_map should have 6 entries total");
}

#[test]
fn test_support_map_no_self_support() {
    // Setup: 2 variables with domains [1,2], [1,2,3]
    let store = Store::new(&[vec![1, 2], vec![1, 2, 3]]);
    let vars = vec![0, 1];
    
    // Call: compute_support_map
    let support_map = pyccp_core::propagators::compute_support_map(&vars, &store);
    
    // For each (var_i, val_a) entry in support_map:
    for (&(var_i, _val_a), supports) in support_map.iter() {
        // For each (var_j, val_b) supporting it:
        for &(var_j, _val_b) in supports {
            // Assert var_i ≠ var_j (no self-support)
            assert_ne!(var_i, var_j, "Variable {} should not support itself", var_i);
        }
    }
}

#[test]
fn test_ac4_detects_no_support() {
    // Setup: domains = [[1,2], [1], [1]]
    // Constraint: AllDiff on all 3 variables
    // Vars 1 and 2 must both be 1 → conflict with AllDiff
    // AC-4 should prune value 1 from var 0 (no support from other vars)
    let mut store = Store::new(&[vec![1, 2], vec![1], vec![1]]);
    let vars = vec![0, 1, 2];
    let mut state = AllDiffState {
        algorithm: AllDiffAlgorithm::AC4,
        vars: vars.clone(),
        support_map: None,
        supports_initialized: false,
    };
    
    let result = propagate_ac4(&mut state, &mut store);
    // AC-4 should succeed in pruning (0, 1) since it has no support
    assert!(result.is_ok(), "AC-4 propagation should succeed");
    // After pruning (0, 1), var 0 should only have domain [2]
    assert_eq!(store.domain(0).to_vec(), &[2], "Var 0 should have domain {{2}} after pruning unsupported value 1");
}

#[test]
fn test_ac4_detects_conflict() {
    // Setup: domains = [[1], [1]]
    // Constraint: AllDiff on 2 variables
    // Both must be 1 → AllDiff violated → conflict
    let mut store = Store::new(&[vec![1], vec![1]]);
    let vars = vec![0, 1];
    let mut state = AllDiffState {
        algorithm: AllDiffAlgorithm::AC4,
        vars: vars.clone(),
        support_map: None,
        supports_initialized: false,
    };
    
    let result = propagate_ac4(&mut state, &mut store);
    assert!(result.is_err(), "AC-4 should detect conflict when two vars must have same value");
}

#[test]
fn test_propagate_dispatcher_ac4_mode() {
    // Small domains [1,2,3] and [1,2,3] → should trigger AC-4
    let store = Store::new(&[vec![1, 2, 3], vec![1, 2, 3]]);
    let mut constraint = AllDiff::new(vec![0, 1], &store);
    
    // Assert: constraint.state.algorithm == AllDiffAlgorithm::AC4
    assert_eq!(
        constraint.state.algorithm as u8,
        AllDiffAlgorithm::AC4 as u8,
        "Small domain constraint should use AC-4"
    );
    
    // Call propagate, assert Ok(())
    let mut store_copy = Store::new(&[vec![1, 2, 3], vec![1, 2, 3]]);
    let result = constraint.propagate(&mut store_copy);
    assert!(result.is_ok(), "Propagate should succeed with AC-4");
}

#[test]
fn test_propagate_dispatcher_simple_mode() {
    // Large domains [0..1001] and [0..1001] → should use simple
    let domain_0: Vec<i64> = (0..=1000).collect();
    let domain_1: Vec<i64> = (0..=1000).collect();
    let store = Store::new(&[domain_0.clone(), domain_1.clone()]);
    let mut constraint = AllDiff::new(vec![0, 1], &store);
    
    // Assert: constraint.state.algorithm == AllDiffAlgorithm::SimpleIntersection
    assert_eq!(
        constraint.state.algorithm as u8,
        AllDiffAlgorithm::SimpleIntersection as u8,
        "Large domain constraint should use simple propagation"
    );
    
    // Call propagate, assert Ok(())
    let mut store_copy = Store::new(&[domain_0, domain_1]);
    let result = constraint.propagate(&mut store_copy);
    assert!(result.is_ok(), "Propagate should succeed with simple mode");
}
