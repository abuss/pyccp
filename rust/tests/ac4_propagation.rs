use pyccp_core::store::Store;
use pyccp_core::propagators::should_use_ac4;

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
