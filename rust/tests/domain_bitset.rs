/// Test suite for the Domain enum (Vec/BitSet hybrid representation)
/// This guides the implementation of the Domain API

#[cfg(test)]
mod domain_tests {
    use pyccp_core::store::Domain;

    #[test]
    fn domain_vec_creation_and_contains() {
        // Test that a Domain::Vec can be created and contains values
        let domain = Domain::new(vec![1, 2, 3, 5, 8]);
        assert!(domain.contains(1));
        assert!(domain.contains(2));
        assert!(domain.contains(5));
        assert!(!domain.contains(4));
        assert!(!domain.contains(9));
    }

    #[test]
    fn domain_remove_value() {
        // Test that remove() works and tracks whether value existed
        let mut domain = Domain::new(vec![1, 2, 3]);
        let removed = domain.remove(2);
        assert!(removed); // 2 was present
        assert!(!domain.contains(2));
        assert!(domain.contains(1));
        assert!(domain.contains(3));
        
        // Removing non-existent value returns false
        let removed_again = domain.remove(2);
        assert!(!removed_again);
    }

    #[test]
    fn domain_len_and_is_empty() {
        // Test that len() and is_empty() work correctly
        let mut domain = Domain::new(vec![10, 20, 30]);
        assert_eq!(domain.len(), 3);
        assert!(!domain.is_empty());
        
        domain.remove(20);
        assert_eq!(domain.len(), 2);
        assert!(!domain.is_empty());
        
        domain.remove(10);
        domain.remove(30);
        assert_eq!(domain.len(), 0);
        assert!(domain.is_empty());
    }

    #[test]
    fn domain_min_max() {
        // Test that min() and max() return correct boundary values
        let domain = Domain::new(vec![5, 15, 25, 35]);
        assert_eq!(domain.min(), Some(5));
        assert_eq!(domain.max(), Some(35));
        
        let domain_single = Domain::new(vec![42]);
        assert_eq!(domain_single.min(), Some(42));
        assert_eq!(domain_single.max(), Some(42));
    }

    #[test]
    fn domain_iter() {
        // Test that iteration yields values in sorted order
        let domain = Domain::new(vec![3, 1, 4, 1, 5]); // duplicates and unsorted
        let values: Vec<i64> = domain.iter().copied().collect();
        let mut expected = vec![1, 3, 4, 5];
        expected.sort();
        expected.dedup();
        assert_eq!(values, expected);
    }
}
