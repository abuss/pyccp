/// Test suite for the MRV (Minimum Remaining Values) cache
/// This tests that the cache correctly returns the variable with smallest unfixed domain

#[cfg(test)]
mod mrv_cache_tests {
    use pyccp_core::store::Store;

    #[test]
    fn cache_returns_same_value() {
        // Test that mrv() returns the same result when called consecutively
        // without any domain changes
        let mut store = Store::new(&[
            vec![1, 2, 3, 4],      // 4 values (unfixed)
            vec![10, 20],          // 2 values (unfixed) <- MRV
            vec![100, 200, 300],   // 3 values (unfixed)
        ]);
        
        let first = store.mrv();
        let second = store.mrv();
        
        assert_eq!(first, second);
        assert_eq!(first, Some(1)); // cell 1 has smallest domain size (2)
    }

    #[test]
    fn cache_invalidated_after_remove() {
        // Test that mrv() cache is invalidated after a domain removal
        let mut store = Store::new(&[
            vec![1, 2, 3, 4],      // 4 values (unfixed)
            vec![10, 20],          // 2 values (unfixed) <- initially MRV
            vec![100, 200, 300],   // 3 values (unfixed)
        ]);
        
        // First call: cell 1 is MRV (size 2)
        let initial = store.mrv();
        assert_eq!(initial, Some(1));
        
        // Remove values from cell 0 to make it size 2
        store.remove(0, 1).unwrap();
        store.remove(0, 2).unwrap();
        // Now cell 0 has [3, 4] (size 2), cell 1 has [10, 20] (size 2)
        // Cache should be invalidated, and both have same size
        // MRV should return cell 0 (first in iteration)
        
        let after_remove = store.mrv();
        assert_eq!(after_remove, Some(0));
    }

    #[test]
    fn mrv_picks_smallest_unfixed() {
        // Test that mrv() correctly picks the unfixed variable with smallest domain
        let mut store = Store::new(&[
            vec![1, 2, 3, 4, 5],   // 5 values (unfixed)
            vec![10, 20, 30],      // 3 values (unfixed)
            vec![100],             // 1 value (fixed)
            vec![200, 300, 400],   // 3 values (unfixed)
            vec![1000, 2000],      // 2 values (unfixed) <- MRV
        ]);
        
        // Cell 2 is fixed, so should not be considered
        assert!(store.is_fixed(2));
        
        // Cell 4 has the smallest unfixed domain (size 2)
        let mrv_cell = store.mrv();
        assert_eq!(mrv_cell, Some(4));
        
        // Fix cell 4, now cell 1 and 3 are tied at size 3, should return 1 (first)
        store.commit(4, 1000).unwrap();
        let new_mrv = store.mrv();
        assert_eq!(new_mrv, Some(1)); // cell 1 and 3 both have 3 values, return first
    }
}
