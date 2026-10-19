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
