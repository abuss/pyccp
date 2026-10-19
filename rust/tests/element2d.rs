use pyccp_core::propagators::{Element2D, Registry};
use pyccp_core::store::Store;

/// Test Element2D: both row and col fixed → result is fixed
#[test]
fn test_element2d_both_fixed() {
    // 3x3 matrix represented by cells with domains
    // Cells 0-8 represent the matrix values
    // row_var fixed to 1, col_var fixed to 2
    // index = 1*3+2 = 5, so result should equal cell 5
    
    let mut s = Store::new(&[
        vec![0],           // cell 0: matrix[0,0]
        vec![1],           // cell 1: matrix[0,1]
        vec![2],           // cell 2: matrix[0,2]
        vec![3],           // cell 3: matrix[1,0]
        vec![4],           // cell 4: matrix[1,1]
        vec![5],           // cell 5: matrix[1,2] <- this is array[5]
        vec![6],           // cell 6: matrix[2,0]
        vec![7],           // cell 7: matrix[2,1]
        vec![8],           // cell 8: matrix[2,2]
        vec![1],           // cell 9: row_var fixed to 1
        vec![2],           // cell 10: col_var fixed to 2
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8],  // cell 11: result_var
    ]);
    
    let linearized_array: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];  // cell indices
    let ncols = 3;
    let row_var = 9;
    let col_var = 10;
    let result_var = 11;
    
    let mut r = Registry::new();
    r.add(Box::new(Element2D::new(linearized_array, row_var, col_var, result_var, ncols)));
    
    r.fixpoint(&mut s).unwrap();
    
    // result_var should be fixed to 5 (the value of cell 5 due to index=5)
    assert!(s.is_fixed(result_var), "result_var should be fixed");
    assert_eq!(s.value(result_var), 5, "result_var should equal cell[5].value = 5");
}

/// Test Element2D: result fixed → invalid (row, col) pairs are pruned
#[test]
fn test_element2d_result_fixed() {
    // 2x2 matrix: cells 0-3
    // result fixed to 20 (cell 1's value) → only (row=0, col=1) valid
    
    let mut s = Store::new(&[
        vec![10],          // cell 0: matrix[0,0]
        vec![20],          // cell 1: matrix[0,1]
        vec![30],          // cell 2: matrix[1,0]
        vec![40],          // cell 3: matrix[1,1]
        vec![0, 1],        // cell 4: row_var ∈ [0, 1]
        vec![0, 1],        // cell 5: col_var ∈ [0, 1]
        vec![20],          // cell 6: result_var fixed to 20
    ]);
    
    let linearized_array: Vec<usize> = vec![0, 1, 2, 3];  // cell indices
    let ncols = 2;
    let row_var = 4;
    let col_var = 5;
    let result_var = 6;
    
    let mut r = Registry::new();
    r.add(Box::new(Element2D::new(linearized_array, row_var, col_var, result_var, ncols)));
    
    r.fixpoint(&mut s).unwrap();
    
    // row_var should be fixed to 0
    assert!(s.is_fixed(row_var), "row_var should be fixed");
    assert_eq!(s.value(row_var), 0);
    
    // col_var should be fixed to 1
    assert!(s.is_fixed(col_var), "col_var should be fixed");
    assert_eq!(s.value(col_var), 1);
}

/// Test Element2D: bounds-based pruning when indices open
#[test]
fn test_element2d_bounds_prune() {
    // 3x3 matrix with specific values on diagonal
    // Only (1,1)→4 and (2,2)→8 in bounds [2,5], so (1,1) is the only valid pair
    
    let mut s = Store::new(&[
        vec![1],           // cell 0
        vec![10],          // cell 1
        vec![10],          // cell 2
        vec![10],          // cell 3
        vec![4],           // cell 4: matrix[1,1]
        vec![10],          // cell 5
        vec![10],          // cell 6
        vec![10],          // cell 7
        vec![8],           // cell 8: matrix[2,2]
        vec![0, 1, 2],     // cell 9: row_var
        vec![0, 1, 2],     // cell 10: col_var
        vec![2, 3, 4, 5],  // cell 11: result_var ∈ [2, 5]
    ]);
    
    let linearized_array: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
    let ncols = 3;
    let row_var = 9;
    let col_var = 10;
    let result_var = 11;
    
    let mut r = Registry::new();
    r.add(Box::new(Element2D::new(linearized_array, row_var, col_var, result_var, ncols)));
    
    r.fixpoint(&mut s).unwrap();
    
    // row_var and col_var should be fixed to (1, 1)
    assert!(s.is_fixed(row_var), "row_var should be fixed to 1");
    assert_eq!(s.value(row_var), 1);
    assert!(s.is_fixed(col_var), "col_var should be fixed to 1");
    assert_eq!(s.value(col_var), 1);
    assert!(s.is_fixed(result_var), "result_var should be fixed to 4");
    assert_eq!(s.value(result_var), 4);
}

/// Test Element2D: out-of-bounds indices cause conflict
#[test]
fn test_element2d_out_of_bounds() {
    let mut s = Store::new(&[
        vec![1],           // cell 0
        vec![2],           // cell 1
        vec![3],           // cell 2
        vec![4],           // cell 3
        vec![5],           // row_var fixed to 5 (out of bounds)
        vec![0],           // col_var fixed to 0
        vec![1, 2, 3, 4],  // result_var
    ]);
    
    let linearized_array: Vec<usize> = vec![0, 1, 2, 3];  // 2x2 matrix
    let ncols = 2;
    let row_var = 4;
    let col_var = 5;
    let result_var = 6;
    
    let mut r = Registry::new();
    r.add(Box::new(Element2D::new(linearized_array, row_var, col_var, result_var, ncols)));
    
    let result = r.fixpoint(&mut s);
    assert!(result.is_err(), "out-of-bounds index should cause conflict");
}

/// Test Element2D: result value not achievable → conflict
#[test]
fn test_element2d_conflict_result() {
    let mut s = Store::new(&[
        vec![1],           // cell 0
        vec![2],           // cell 1
        vec![3],           // cell 2
        vec![4],           // cell 3
        vec![0, 1],        // row_var
        vec![0, 1],        // col_var
        vec![99],          // result_var fixed to 99 (not in matrix)
    ]);
    
    let linearized_array: Vec<usize> = vec![0, 1, 2, 3];
    let ncols = 2;
    let row_var = 4;
    let col_var = 5;
    let result_var = 6;
    
    let mut r = Registry::new();
    r.add(Box::new(Element2D::new(linearized_array, row_var, col_var, result_var, ncols)));
    
    let result = r.fixpoint(&mut s);
    assert!(result.is_err(), "result value not achievable should cause conflict");
}

/// Test Element2D: 3x3 matrix with mixed variable bindings
#[test]
fn test_element2d_3x3_mixed() {
    // 3x3 matrix with diagonal values [0, 5, 10]
    // row ∈ [1, 2], col ∈ [1, 2], result ∈ [5, 10]
    // Valid: (1, 1)→5 and (2, 2)→10
    
    let mut s = Store::new(&[
        vec![0],           // cell 0
        vec![10],          // cell 1
        vec![20],          // cell 2
        vec![30],          // cell 3
        vec![5],           // cell 4: matrix[1,1]
        vec![40],          // cell 5
        vec![50],          // cell 6
        vec![60],          // cell 7
        vec![10],          // cell 8: matrix[2,2]
        vec![1, 2],        // cell 9: row_var ∈ [1, 2]
        vec![1, 2],        // cell 10: col_var ∈ [1, 2]
        vec![5, 10],       // cell 11: result_var ∈ [5, 10]
    ]);
    
    let linearized_array: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
    let ncols = 3;
    let row_var = 9;
    let col_var = 10;
    let result_var = 11;
    
    let mut r = Registry::new();
    r.add(Box::new(Element2D::new(linearized_array, row_var, col_var, result_var, ncols)));
    
    r.fixpoint(&mut s).unwrap();
    
    // Domains should remain non-empty with multiple solutions possible
    let row_dom = s.domain(row_var);
    let col_dom = s.domain(col_var);
    let res_dom = s.domain(result_var);
    
    assert!(!row_dom.is_empty(), "row_var domain should not be empty");
    assert!(!col_dom.is_empty(), "col_var domain should not be empty");
    assert!(!res_dom.is_empty(), "result_var domain should not be empty");
    
    // Both (1, 1, 5) and (2, 2, 10) should be possible
    assert!(row_dom.contains(1), "row should contain 1");
    assert!(row_dom.contains(2), "row should contain 2");
    assert!(col_dom.contains(1), "col should contain 1");
    assert!(col_dom.contains(2), "col should contain 2");
}

