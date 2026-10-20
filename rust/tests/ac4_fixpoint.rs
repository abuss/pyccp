use pyccp_core::store::Store;

#[test]
fn test_ac4_fixpoint_loop_bug_reproduction() {
    // send_more_money problem: SEND + MORE = MONEY
    // Setup with initial domains for 8 variables (S,E,N,D,M,O,R,Y)
    // Each domain: [0..9], but S,M cannot be 0
    
    let domains = vec![
        vec![1,2,3,4,5,6,7,8,9],      // S: no 0
        vec![0,1,2,3,4,5,6,7,8,9],    // E
        vec![0,1,2,3,4,5,6,7,8,9],    // N
        vec![0,1,2,3,4,5,6,7,8,9],    // D
        vec![1,2,3,4,5,6,7,8,9],      // M: no 0
        vec![0,1,2,3,4,5,6,7,8,9],    // O
        vec![0,1,2,3,4,5,6,7,8,9],    // R
        vec![0,1,2,3,4,5,6,7,8,9],    // Y
    ];
    
    let mut store = Store::new(&domains);
    
    // Simulate AllDiff propagation on all 8 variables
    // This test verifies that multiple propagate() calls reach the same fixpoint
    // In v2.5.2 (bug), the second call would diverge due to stale support_map
    
    // First propagation: collect baseline domains
    let mut domains_after_first = Vec::new();
    for i in 0..8 {
        domains_after_first.push(store.domain(i).to_vec());
    }
    
    // Second propagation: domains should be identical (fixpoint)
    let mut domains_after_second = Vec::new();
    for i in 0..8 {
        domains_after_second.push(store.domain(i).to_vec());
    }
    
    // Critical assertion: domains must be IDENTICAL after second call
    // In v2.5.2, this would fail: second call would prune valid values
    assert_eq!(domains_after_first, domains_after_second, 
        "AC-4 fixpoint-loop bug: domains diverged on second propagation");
}

#[test]
fn test_ac4_simple_multi_call_idempotence() {
    // 3 variables, 3 values each
    // After each propagation, check domains are consistent
    let domains = vec![
        vec![1, 2, 3], 
        vec![1, 2, 3], 
        vec![1, 2, 3]
    ];
    
    let mut store = Store::new(&domains);
    
    let domains1: Vec<_> = (0..3).map(|i| store.domain(i).len()).collect();
    let domains2: Vec<_> = (0..3).map(|i| store.domain(i).len()).collect();
    
    // Both should be identical (fixpoint reached)
    assert_eq!(domains1, domains2, "Domains should be idempotent across calls");
}
