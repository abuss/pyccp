use crate::store::{Conflict, Store};

pub trait Propagator: Send + Sync {
    fn cells(&self) -> &[usize];
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict>;
}

pub struct NeqLinear { cells: Vec<usize>, terms: Vec<(usize, i64)>, k: i64 }
impl NeqLinear {
    pub fn new(terms: Vec<(usize, i64)>, k: i64) -> NeqLinear {
        NeqLinear { cells: terms.iter().map(|t| t.0).collect(), terms, k }
    }

    /// Compute min/max possible sum for open (unfixed) terms.
    /// Follows LinearEq::compute_bounds pattern.
    fn compute_bounds_for_open(&self, store: &Store, open_terms: &[(usize, i64)], fixed_sum: i64) -> (i64, i64) {
        let mut min_open = 0i64;
        let mut max_open = 0i64;
        
        for (var, coeff) in open_terms {
            let bounds = store.bounds(*var);
            if *coeff > 0 {
                min_open += coeff * bounds.min;
                max_open += coeff * bounds.max;
            } else if *coeff < 0 {
                min_open += coeff * bounds.max;
                max_open += coeff * bounds.min;
            }
        }
        
        (fixed_sum + min_open, fixed_sum + max_open)
    }
}
impl Propagator for NeqLinear {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // Collect open (unfixed) and fixed terms
        let open: Vec<(usize, i64)> = self.terms.iter()
            .filter(|(c, _)| !store.is_fixed(*c)).copied().collect();
        let mut fixed_sum = 0i64;
        for (c, k) in &self.terms {
            if store.is_fixed(*c) { fixed_sum += k * store.value(*c); }
        }
        
        // No open terms: check if fixed sum == k (conflict if true)
        if open.is_empty() {
            return if fixed_sum == self.k { Err(Conflict) } else { Ok(()) };
        }
        
        // Exactly one open term: direct solve
        if open.len() == 1 {
            let (c, coef) = open[0];
            if coef == 0 { return Err(Conflict); }
            let rem = self.k - fixed_sum;
            if rem % coef == 0 {
                let v = rem / coef;
                if store.domain(c).contains(v) { store.remove(c, v)?; }
            }
            return Ok(());
        }
        
        // Multiple open terms: bounds-based reasoning
        let (min_sum, max_sum) = self.compute_bounds_for_open(store, &open, fixed_sum);
        
        // Early conflict: if we can only achieve sum == k, the constraint fails
        // This only happens if min_sum == max_sum == k
        if min_sum == max_sum && min_sum == self.k {
            return Err(Conflict);
        }
        
        // If k is not in [min_sum, max_sum], the constraint is automatically satisfied
        // (we can't make sum == k, so != is always true)
        
        Ok(())
    }
}

pub struct AllDiff { cells: Vec<usize> }
impl AllDiff {
    pub fn new(cells: Vec<usize>) -> AllDiff { AllDiff { cells } }

    /// AC-3 algorithm for AllDiff constraint.
    /// 
    /// Phase 1: Remove values from fixed variables
    /// - For each fixed variable with value v, remove v from all other variables
    /// 
    /// Phase 2: Check support and fix unique supporters
    /// - For each value v in the union of all domains:
    ///   - Count unfixed variables that have v in their domain
    ///   - If count == 0 and a fixed variable has v: conflict
    ///   - If count == 1: fix that variable to v
    fn collect_all_values(&self, store: &Store) -> Vec<i64> {
        let mut values = std::collections::HashSet::new();
        for &var in &self.cells {
            for &val in store.domain(var).iter() {
                values.insert(val);
            }
        }
        let mut sorted: Vec<i64> = values.into_iter().collect();
        sorted.sort();
        sorted
    }
}
impl Propagator for AllDiff {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // Phase 1: Remove fixed values from domains
        for &var_i in &self.cells {
            if store.is_fixed(var_i) {
                let value = store.value(var_i);
                for &var_j in &self.cells {
                    if var_i != var_j {
                        store.remove(var_j, value)?;
                    }
                }
            }
        }
        
        // Phase 2: Check for unsupported values and fix unique supporters
        for value in self.collect_all_values(store) {
            // Count supporters: both fixed variables assigned v and unfixed variables with v
            let supporters: Vec<usize> = self.cells.iter()
                .filter(|&&var| {
                    if store.is_fixed(var) {
                        store.value(var) == value
                    } else {
                        store.domain(var).contains(value)
                    }
                })
                .copied()
                .collect();
            
            match supporters.len() {
                0 => {
                    // Value v has no support (not used by any variable)
                    // This is not a conflict - it just means v won't be assigned
                }
                1 => {
                    // Value v can only go to one variable; fix it if not already fixed
                    let var = supporters[0];
                    if !store.is_fixed(var) {
                        store.commit(var, value)?;
                    }
                }
                _ => {
                    // Multiple supporters; ok (no more pruning needed here)
                }
            }
        }
        
        Ok(())
    }
}

pub struct LinearEq { cells: Vec<usize>, terms: Vec<(usize, i64)>, rhs: i64 }
impl LinearEq {
    pub fn new(terms: Vec<(usize, i64)>, rhs: i64) -> LinearEq {
        LinearEq { cells: terms.iter().map(|t| t.0).collect(), terms, rhs }
    }

    /// Compute min/max possible sum across all variables in the constraint.
    /// For each term (var, coeff):
    ///   - If coeff > 0: use bounds.min for min_sum, bounds.max for max_sum
    ///   - If coeff < 0: use bounds.max for min_sum, bounds.min for max_sum (reversed)
    fn compute_bounds(&self, store: &Store) -> (i64, i64) {
        let mut min_sum = 0i64;
        let mut max_sum = 0i64;
        
        for (var, coeff) in &self.terms {
            let bounds = store.bounds(*var);
            if *coeff > 0 {
                min_sum += coeff * bounds.min;
                max_sum += coeff * bounds.max;
            } else if *coeff < 0 {
                // For negative coefficients, the relationship is reversed
                min_sum += coeff * bounds.max;
                max_sum += coeff * bounds.min;
            }
            // If coeff == 0, contributes 0 to both sums
        }
        
        (min_sum, max_sum)
    }
}

impl Propagator for LinearEq {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // First: bounds-based early conflict detection
        let (min_sum, max_sum) = self.compute_bounds(store);
        if min_sum > self.rhs || max_sum < self.rhs {
            return Err(Conflict);
        }

        // Original propagation: when all-but-one term is fixed
        let open: Vec<(usize, i64)> = self.terms.iter()
            .filter(|(c, _)| !store.is_fixed(*c)).copied().collect();
        if open.len() > 1 { return Ok(()); }
        let mut sum = self.rhs;
        for (c, k) in &self.terms {
            if store.is_fixed(*c) { sum -= k * store.value(*c); }
        }
        if open.len() == 1 {
            let (c, k) = open[0];
            if k == 0 { return Err(Conflict); }
            if sum % k != 0 { return Err(Conflict); }
            let v = sum / k;
            for cur in store.domain(c).to_vec() {
                if cur != v { store.remove(c, cur)?; }
            }
        } else {
            let mut total = 0;
            for (c, k) in &self.terms { total += k * store.value(*c); }
            if total != self.rhs { return Err(Conflict); }
        }
        Ok(())
    }
}

pub struct LeLinear { cells: Vec<usize>, terms: Vec<(usize, i64)>, rhs: i64 }
impl LeLinear {
    pub fn new(terms: Vec<(usize, i64)>, rhs: i64) -> LeLinear {
        LeLinear { cells: terms.iter().map(|t| t.0).collect(), terms, rhs }
    }

    /// Compute min/max possible sum across all variables in the constraint.
    /// Similar to LinearEq: we need sum <= rhs, so min_sum is always achievable
    /// and max_sum gives us the tightest upper bound.
    fn compute_bounds(&self, store: &Store) -> (i64, i64) {
        let mut min_sum = 0i64;
        let mut max_sum = 0i64;
        
        for (var, coeff) in &self.terms {
            let bounds = store.bounds(*var);
            if *coeff > 0 {
                min_sum += coeff * bounds.min;
                max_sum += coeff * bounds.max;
            } else if *coeff < 0 {
                min_sum += coeff * bounds.max;
                max_sum += coeff * bounds.min;
            }
        }
        
        (min_sum, max_sum)
    }
}

impl Propagator for LeLinear {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // For sum <= rhs: conflict if min_sum > rhs
        let (min_sum, max_sum) = self.compute_bounds(store);
        if min_sum > self.rhs {
            return Err(Conflict);
        }
        // If max_sum <= rhs, constraint is always satisfied; no pruning needed
        if max_sum <= self.rhs {
            return Ok(());
        }
        
        // Bounds-based pruning: for each term (var, coeff),
        // compute the maximum value it can take without violating the constraint
        // Constraint: sum(coeff_i * var_i) <= rhs
        // So for term j: coeff_j * var_j <= rhs - sum_{i≠j}(coeff_i * var_i)
        
        for (j, (var, coeff)) in self.terms.iter().enumerate() {
            if *coeff == 0 { continue; }
            
            let bounds = store.bounds(*var);
            
            // Compute max sum of all OTHER variables
            let mut other_max = 0i64;
            for (i, (other_var, other_coeff)) in self.terms.iter().enumerate() {
                if i == j { continue; }
                let other_bounds = store.bounds(*other_var);
                if *other_coeff > 0 {
                    other_max += other_coeff * other_bounds.max;
                } else if *other_coeff < 0 {
                    other_max += other_coeff * other_bounds.min;
                }
            }
            
            // Maximum value for this variable: coeff * var <= rhs - other_max
            // So var <= (rhs - other_max) / coeff
            // But we need to be careful with negative coefficients
            
            let space = self.rhs - other_max;
            
            let (new_min, new_max) = if *coeff > 0 {
                // Positive coefficient: var <= space / coeff
                let max_val = space / *coeff;
                (bounds.min, max_val)
            } else {
                // Negative coefficient: var >= space / coeff (division flips inequality)
                let min_val = space / *coeff;
                (min_val, bounds.max)
            };
            
            // Prune domain
            if new_max < bounds.max {
                for v in store.domain(*var).to_vec() {
                    if v > new_max { store.remove(*var, v)?; }
                }
            }
            if new_min > bounds.min {
                for v in store.domain(*var).to_vec() {
                    if v < new_min { store.remove(*var, v)?; }
                }
            }
        }
        
        Ok(())
    }
}

pub struct NonZero { cells: [usize; 1] }
impl NonZero {
    pub fn new(i: usize) -> NonZero { NonZero { cells: [i] } }
}
impl Propagator for NonZero {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        store.remove(self.cells[0], 0)
    }
}

pub struct LessThan { cells: [usize; 2] }
impl LessThan { pub fn new(i: usize, j: usize) -> LessThan { LessThan { cells: [i, j] } } }
impl Propagator for LessThan {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        let [i, j] = self.cells;
        
        // Get bounds for left (i) and right (j)
        let left_bounds = store.bounds(i);
        let right_bounds = store.bounds(j);
        
        // Early conflict check: if left.min >= right.max, constraint is unsatisfiable
        if left_bounds.min >= right_bounds.max {
            return Err(Conflict);
        }
        
        // Tighten left variable: i < j means i <= j.max - 1
        let new_left_max = right_bounds.max - 1;
        if new_left_max < left_bounds.max {
            store.tighten_bounds(i, crate::store::Bounds::new(left_bounds.min, new_left_max))?;
            // Prune domain values outside tightened bounds
            for v in store.domain(i).to_vec() {
                if v > new_left_max { store.remove(i, v)?; }
            }
        }
        
        // Tighten right variable: i < j means j >= i.min + 1
        let new_right_min = left_bounds.min + 1;
        if new_right_min > right_bounds.min {
            store.tighten_bounds(j, crate::store::Bounds::new(new_right_min, right_bounds.max))?;
            // Prune domain values outside tightened bounds
            for v in store.domain(j).to_vec() {
                if v < new_right_min { store.remove(j, v)?; }
            }
        }
        
        // Domain-based pruning for fixed values (keep existing behavior for compatibility)
        if store.is_fixed(i) && store.is_fixed(j) {
            return if store.value(i) < store.value(j) { Ok(()) } else { Err(Conflict) };
        }
        if store.is_fixed(i) {
            let v = store.value(i);
            for d in store.domain(j).to_vec() { if d <= v { store.remove(j, d)?; } }
        }
        if store.is_fixed(j) {
            let v = store.value(j);
            for d in store.domain(i).to_vec() { if d >= v { store.remove(i, d)?; } }
        }
        Ok(())
    }
}

pub struct Multiply { cells: [usize; 2], k: i64 }
impl Multiply { pub fn new(i: usize, j: usize, k: i64) -> Multiply { Multiply { cells: [i, j], k } } }
impl Propagator for Multiply {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        let [i, j] = self.cells;
        
        // Both fixed: verify constraint
        if store.is_fixed(i) && store.is_fixed(j) {
            return if store.value(i) * store.value(j) == self.k { Ok(()) } else { Err(Conflict) };
        }
        
        // One fixed: original all-but-one logic
        for (a, b) in [(i, j), (j, i)] {
            if store.is_fixed(a) {
                let v = store.value(a);
                if v == 0 { if self.k != 0 { return Err(Conflict); } continue; }
                if self.k % v != 0 { return Err(Conflict); }
                store.commit(b, self.k / v)?;
            }
        }
        
        // Both open: bounds-based divisibility reasoning
        // Only values that divide k are valid (necessary for x * y = k)
        if !store.is_fixed(i) && !store.is_fixed(j) {
            // Prune i: only divisors of k are valid
            let i_domain: Vec<i64> = store.domain(i).to_vec();
            for &val in &i_domain {
                if val == 0 {
                    if self.k != 0 { store.remove(i, val)?; }
                } else if self.k % val != 0 {
                    store.remove(i, val)?;
                }
            }
            
            // Prune j: only divisors of k are valid
            let j_domain: Vec<i64> = store.domain(j).to_vec();
            for &val in &j_domain {
                if val == 0 {
                    if self.k != 0 { store.remove(j, val)?; }
                } else if self.k % val != 0 {
                    store.remove(j, val)?;
                }
            }
        }
        
        Ok(())
    }
}

pub struct Element { cells: Vec<usize>, array: Vec<usize>, idx: usize, y: usize }
impl Element {
    pub fn new(array: Vec<usize>, idx: usize, y: usize) -> Element {
        let mut cells = array.clone();
        cells.push(idx);
        cells.push(y);
        Element { cells, array, idx, y }
    }
}
impl Propagator for Element {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // idx is fixed: y = array[idx]
        if store.is_fixed(self.idx) {
            let idx = store.value(self.idx);
            if idx < 0 || idx as usize >= self.array.len() { return Err(Conflict); }
            let c = self.array[idx as usize];
            if store.is_fixed(c) {
                store.commit(self.y, store.value(c))?;
            } else {
                let d = store.domain(c).to_vec();
                for v in store.domain(self.y).to_vec() { if !d.contains(&v) { store.remove(self.y, v)?; } }
            }
            return Ok(());
        }
        
        // y is fixed: which indices have array[idx] == y?
        if store.is_fixed(self.y) {
            let yv = store.value(self.y);
            for (i, c) in self.array.iter().enumerate() {
                if store.is_fixed(*c) {
                    if store.value(*c) != yv { store.remove(self.idx, i as i64)?; }
                    else { store.commit(self.idx, i as i64)?; }
                }
            }
            return Ok(());
        }
        
        // Both open: bounds-based index filtering
        // Use y's bounds to prune indices where array[idx] cannot be in y's range
        let y_bounds = store.bounds(self.y);
        let idx_domain: Vec<i64> = store.domain(self.idx).to_vec();
        
        for &idx_val in &idx_domain {
            if idx_val < 0 || idx_val as usize >= self.array.len() {
                store.remove(self.idx, idx_val)?;
                continue;
            }
            let arr_cell = self.array[idx_val as usize];
            let arr_bounds = store.bounds(arr_cell);
            
            // If array[idx] cannot intersect with y's bounds, prune this index
            // array[idx] ∈ [arr_bounds.min, arr_bounds.max]
            // y ∈ [y_bounds.min, y_bounds.max]
            // If no intersection, this index is invalid
            if arr_bounds.max < y_bounds.min || arr_bounds.min > y_bounds.max {
                store.remove(self.idx, idx_val)?;
            }
        }
        
        Ok(())
    }
}

pub struct Element2D { cells: Vec<usize>, array: Vec<usize>, row_var: usize, col_var: usize, result_var: usize, ncols: usize }
impl Element2D {
    pub fn new(array: Vec<usize>, row_var: usize, col_var: usize, result_var: usize, ncols: usize) -> Element2D {
        let mut cells = array.clone();
        cells.push(row_var);
        cells.push(col_var);
        cells.push(result_var);
        Element2D { cells, array, row_var, col_var, result_var, ncols }
    }
}
impl Propagator for Element2D {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        let nrows = self.array.len() / self.ncols;
        
        // Both row_var and col_var fixed: compute index and fix result_var
        if store.is_fixed(self.row_var) && store.is_fixed(self.col_var) {
            let row = store.value(self.row_var) as usize;
            let col = store.value(self.col_var) as usize;
            
            if row >= nrows || col >= self.ncols {
                return Err(Conflict);
            }
            
            let index = row * self.ncols + col;
            let array_cell = self.array[index];
            
            if store.is_fixed(array_cell) {
                store.commit(self.result_var, store.value(array_cell))?;
            } else {
                let arr_domain = store.domain(array_cell).to_vec();
                for v in store.domain(self.result_var).to_vec() {
                    if !arr_domain.contains(&v) {
                        store.remove(self.result_var, v)?;
                    }
                }
            }
            return Ok(());
        }
        
        // Result_var fixed: prune (row, col) pairs where array[row*ncols+col] != result_val
        if store.is_fixed(self.result_var) {
            let result_val = store.value(self.result_var);
            
            let row_domain: Vec<i64> = store.domain(self.row_var).to_vec();
            for row_val in row_domain {
                let row_usize = row_val as usize;
                if row_usize >= nrows {
                    store.remove(self.row_var, row_val)?;
                    continue;
                }
                
                let col_domain: Vec<i64> = store.domain(self.col_var).to_vec();
                let mut found_valid_col = false;
                for col_val in col_domain {
                    let col_usize = col_val as usize;
                    if col_usize >= self.ncols {
                        store.remove(self.col_var, col_val)?;
                        continue;
                    }
                    
                    let index = row_usize * self.ncols + col_usize;
                    let array_cell = self.array[index];
                    
                    if store.is_fixed(array_cell) {
                        if store.value(array_cell) == result_val {
                            found_valid_col = true;
                        }
                    } else {
                        if store.domain(array_cell).contains(result_val) {
                            found_valid_col = true;
                        }
                    }
                }
                
                if !found_valid_col && store.domain(self.row_var).contains(row_val) {
                    store.remove(self.row_var, row_val)?;
                }
            }
            
            // Also check columns: for each col, is there a valid row?
            let col_domain: Vec<i64> = store.domain(self.col_var).to_vec();
            for col_val in col_domain {
                let col_usize = col_val as usize;
                if col_usize >= self.ncols {
                    store.remove(self.col_var, col_val)?;
                    continue;
                }
                
                let row_domain: Vec<i64> = store.domain(self.row_var).to_vec();
                let mut found_valid_row = false;
                for row_val in row_domain {
                    let row_usize = row_val as usize;
                    if row_usize >= nrows {
                        continue;
                    }
                    
                    let index = row_usize * self.ncols + col_usize;
                    let array_cell = self.array[index];
                    
                    if store.is_fixed(array_cell) {
                        if store.value(array_cell) == result_val {
                            found_valid_row = true;
                        }
                    } else {
                        if store.domain(array_cell).contains(result_val) {
                            found_valid_row = true;
                        }
                    }
                }
                
                if !found_valid_row && store.domain(self.col_var).contains(col_val) {
                    store.remove(self.col_var, col_val)?;
                }
            }
            return Ok(());
        }
        
        // Both open: bounds-based pruning
        let result_bounds = store.bounds(self.result_var);
        
        let row_domain: Vec<i64> = store.domain(self.row_var).to_vec();
        for row_val in row_domain {
            let row_usize = row_val as usize;
            if row_usize >= nrows {
                store.remove(self.row_var, row_val)?;
                continue;
            }
            
            let col_domain: Vec<i64> = store.domain(self.col_var).to_vec();
            let mut found_valid_col = false;
            for col_val in col_domain {
                let col_usize = col_val as usize;
                if col_usize >= self.ncols {
                    store.remove(self.col_var, col_val)?;
                    continue;
                }
                
                let index = row_usize * self.ncols + col_usize;
                let array_cell = self.array[index];
                let arr_bounds = store.bounds(array_cell);
                
                // If array[row*ncols+col] can intersect with result_bounds, keep this col
                if !(arr_bounds.max < result_bounds.min || arr_bounds.min > result_bounds.max) {
                    found_valid_col = true;
                }
            }
            
            // If no valid col for this row, remove row
            if !found_valid_col && store.domain(self.row_var).contains(row_val) {
                store.remove(self.row_var, row_val)?;
            }
        }
        
        // Also check columns: for each col, is there a valid row?
        let col_domain: Vec<i64> = store.domain(self.col_var).to_vec();
        for col_val in col_domain {
            let col_usize = col_val as usize;
            if col_usize >= self.ncols {
                store.remove(self.col_var, col_val)?;
                continue;
            }
            
            let row_domain: Vec<i64> = store.domain(self.row_var).to_vec();
            let mut found_valid_row = false;
            for row_val in row_domain {
                let row_usize = row_val as usize;
                if row_usize >= nrows {
                    continue;
                }
                
                let index = row_usize * self.ncols + col_usize;
                let array_cell = self.array[index];
                let arr_bounds = store.bounds(array_cell);
                
                if !(arr_bounds.max < result_bounds.min || arr_bounds.min > result_bounds.max) {
                    found_valid_row = true;
                }
            }
            
            if !found_valid_row && store.domain(self.col_var).contains(col_val) {
                store.remove(self.col_var, col_val)?;
            }
        }
        
        Ok(())
    }
}

pub struct Cardinality { cells: Vec<usize>, value: i64, count: usize }
impl Cardinality {
    pub fn new(cells: Vec<usize>, value: i64, count: usize) -> Cardinality {
        Cardinality { cells, value, count }
    }
}
impl Propagator for Cardinality {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // ponytail: counts fixed/open cells only; no per-value bounds reasoning.
        let fixed_eq = self.cells.iter().filter(|&c| store.is_fixed(*c) && store.value(*c) == self.value).count();
        let open = self.cells.iter().filter(|&c| !store.is_fixed(*c)).count();
        if fixed_eq > self.count || fixed_eq + open < self.count { return Err(Conflict); }
        if fixed_eq == self.count {
            for c in self.cells.clone() { if !store.is_fixed(c) { store.remove(c, self.value)?; } }
        } else if fixed_eq + open == self.count {
            for c in self.cells.clone() { if !store.is_fixed(c) { store.commit(c, self.value)?; } }
        }
        Ok(())
    }
}

pub struct Modulo { cells: [usize; 1], m: i64, k: i64 }
impl Modulo { pub fn new(c: usize, m: i64, k: i64) -> Modulo { Modulo { cells: [c], m, k } } }
impl Propagator for Modulo {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // ponytail: full domain scan, no ceiling issue — single cell, exact.
        if self.m == 0 { return Err(Conflict); }
        for v in store.domain(self.cells[0]).to_vec() {
            if v.rem_euclid(self.m) != self.k.rem_euclid(self.m) { store.remove(self.cells[0], v)?; }
        }
        Ok(())
    }
}

pub struct And { cells: [usize; 3] }
impl And { pub fn new(x: usize, y: usize, z: usize) -> And { And { cells: [x, y, z] } } }
impl Propagator for And {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // 0/1 domains: z = x ∧ y
        // ponytail: prunes only when enough inputs are fixed to force a value.
        let [x, y, z] = self.cells;
        if store.is_fixed(z) && store.value(z) == 1 {
            store.commit(x, 1)?; store.commit(y, 1)?;
        } else if (store.is_fixed(x) && store.value(x) == 0) || (store.is_fixed(y) && store.value(y) == 0) {
            store.commit(z, 0)?;
        } else if store.is_fixed(x) && store.is_fixed(y) {
            store.commit(z, store.value(x) * store.value(y))?;
        }
        Ok(())
    }
}

pub struct Or { cells: [usize; 3] }
impl Or { pub fn new(x: usize, y: usize, z: usize) -> Or { Or { cells: [x, y, z] } } }
impl Propagator for Or {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // 0/1 domains: z = x ∨ y
        // ponytail: prunes only when enough inputs are fixed to force a value.
        let [x, y, z] = self.cells;
        if store.is_fixed(z) && store.value(z) == 0 {
            store.commit(x, 0)?; store.commit(y, 0)?;
        } else if (store.is_fixed(x) && store.value(x) == 1) || (store.is_fixed(y) && store.value(y) == 1) {
            store.commit(z, 1)?;
        } else if store.is_fixed(x) && store.is_fixed(y) {
            store.commit(z, (store.value(x) + store.value(y)).min(1))?;
        }
        Ok(())
    }
}

pub struct Xor { cells: [usize; 3] }
impl Xor { pub fn new(x: usize, y: usize, z: usize) -> Xor { Xor { cells: [x, y, z] } } }
impl Propagator for Xor {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // 0/1 domains: z = x ⊕ y
        // ponytail: prunes only when two of the three are fixed.
        let [x, y, z] = self.cells;
        if store.is_fixed(x) && store.is_fixed(y) { store.commit(z, store.value(x) ^ store.value(y))?; }
        else if store.is_fixed(x) && store.is_fixed(z) { store.commit(y, store.value(x) ^ store.value(z))?; }
        else if store.is_fixed(y) && store.is_fixed(z) { store.commit(x, store.value(y) ^ store.value(z))?; }
        Ok(())
    }
}

pub struct Registry {
    props: Vec<Box<dyn Propagator>>,
    propagator_index: std::collections::HashMap<usize, Vec<usize>>,  // var_id -> [prop_ids]
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            props: Vec::new(),
            propagator_index: std::collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, p: Box<dyn Propagator>) {
        let prop_id = self.props.len();
        // Index the propagator by its variables
        for &var in p.cells() {
            self.propagator_index.entry(var)
                .or_insert_with(Vec::new)
                .push(prop_id);
        }
        self.props.push(p);
    }

    pub fn propagators_involving(&self, cell: usize) -> Vec<usize> {
        self.propagator_index.get(&cell).cloned().unwrap_or_default()
    }

    /// Get propagator indices that touch any of the given variables
    pub fn propagators_for_vars(&self, vars: &[usize]) -> Vec<usize> {
        let mut propagators = Vec::new();
        for &var in vars {
            if let Some(props) = self.propagator_index.get(&var) {
                propagators.extend_from_slice(props);
            }
        }
        // Remove duplicates while preserving order
        propagators.sort_unstable();
        propagators.dedup();
        propagators
    }

    pub fn fixpoint(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // First iteration: run all propagators
        let mut first_iteration = true;
        
        loop {
            let changed_vars = if first_iteration {
                // First time: mark all variables as changed to run all propagators
                (0..store.num_variables()).collect::<Vec<_>>()
            } else {
                store.get_changed_vars()
            };
            
            if changed_vars.is_empty() {
                break;  // No more changes, fixpoint reached
            }
            
            store.clear_changed_vars();
            first_iteration = false;
            
            // Only propagate affected propagators
            let affected_props = self.propagators_for_vars(&changed_vars);
            
            for prop_id in affected_props {
                self.props[prop_id].propagate(store)?;
            }
        }
        
        store.invalidate_mrv_cache();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neq_linear_prunes_when_other_fixed() {
        // x - y != 0, y fixed to 1 -> remove 1 from x
        let mut s = Store::new(&[vec![1, 2], vec![1]]);
        let mut r = Registry::new();
        r.add(Box::new(NeqLinear::new(vec![(0, 1), (1, -1)], 0)));
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(0).to_vec(), &[2]);
    }

    #[test]
    fn neq_linear_conflict_when_violated() {
        let mut s = Store::new(&[vec![1], vec![1]]);
        let mut r = Registry::new();
        r.add(Box::new(NeqLinear::new(vec![(0, 1), (1, -1)], 0)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn neq_linear_prunes_diagonal() {
        // q0 - q1 != -1; q0 fixed to 2 -> forced q1 value is 3, prune it
        let mut s = Store::new(&[vec![2], vec![1, 3]]);
        let mut r = Registry::new();
        r.add(Box::new(NeqLinear::new(vec![(0, 1), (1, -1)], -1)));
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(1).to_vec(), &[1]);
    }

    #[test]
    fn all_diff_prunes() {
        let mut s = Store::new(&[vec![1, 2], vec![1, 2], vec![1, 2, 3]]);
        let mut r = Registry::new();
        r.add(Box::new(AllDiff::new(vec![0, 1, 2])));
        s.commit(0, 1).unwrap();
        s.commit(1, 2).unwrap();
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(2).to_vec(), &[3]);
    }

    #[test]
    fn linear_eq_propagates() {
        // y - x == 1, x fixed to 1 -> y fixed to 2
        let mut s = Store::new(&[vec![1, 2], vec![2, 3]]);
        let mut r = Registry::new();
        r.add(Box::new(LinearEq::new(vec![(1, 1), (0, -1)], 1)));
        s.commit(0, 1).unwrap();
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(1).to_vec(), &[2]);
    }

    #[test]
    fn neq_linear_zero_coef_is_conflict() {
        let mut s = Store::new(&[vec![1, 2]]);
        let mut r = Registry::new();
        r.add(Box::new(NeqLinear::new(vec![(0, 0)], 1)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn linear_eq_zero_coef_is_conflict() {
        let mut s = Store::new(&[vec![1, 2]]);
        let mut r = Registry::new();
        r.add(Box::new(LinearEq::new(vec![(0, 0)], 1)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn nonzero_prunes() {
        let mut s = Store::new(&[vec![0, 1, 2]]);
        let mut r = Registry::new();
        r.add(Box::new(NonZero::new(0)));
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(0).to_vec(), &[1, 2]);
    }

    #[test]
    fn less_than_prunes() {
        let mut s = Store::new(&[vec![1, 2, 3], vec![1, 2, 3]]);
        let mut r = Registry::new();
        r.add(Box::new(LessThan::new(0, 1)));
        s.commit(1, 2).unwrap();
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(0).to_vec(), &[1]);
    }

    #[test]
    fn less_than_conflict() {
        let mut s = Store::new(&[vec![2], vec![2]]);
        let mut r = Registry::new();
        r.add(Box::new(LessThan::new(0, 1)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn multiply_prunes() {
        let mut s = Store::new(&[vec![2, 3], vec![4, 6]]);
        let mut r = Registry::new();
        r.add(Box::new(Multiply::new(0, 1, 12)));
        s.commit(0, 2).unwrap();
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(1).to_vec(), &[6]);
    }

    #[test]
    fn multiply_conflict() {
        let mut s = Store::new(&[vec![2], vec![5]]);
        let mut r = Registry::new();
        r.add(Box::new(Multiply::new(0, 1, 12)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn element_prunes() {
        // y == array[idx]; array = cells 0,1,2; idx = cell 3; y = cell 4
        let mut s = Store::new(&[vec![1], vec![2], vec![3], vec![0, 1], vec![1, 3]]);
        let mut r = Registry::new();
        r.add(Box::new(Element::new(vec![0, 1, 2], 3, 4)));
        s.commit(3, 0).unwrap();          // idx -> array[0] = 1
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(4).to_vec(), &[1]);
    }

    #[test]
    fn element_conflict() {
        let mut s = Store::new(&[vec![1], vec![2], vec![0], vec![0], vec![2]]);
        let mut r = Registry::new();
        r.add(Box::new(Element::new(vec![0, 1, 2], 3, 4)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn cardinality_prunes() {
        let mut s = Store::new(&[vec![3, 4], vec![3, 4], vec![3, 4]]);
        let mut r = Registry::new();
        r.add(Box::new(Cardinality::new(vec![0, 1, 2], 3, 1)));
        s.commit(0, 3).unwrap();          // one equal -> others cannot be 3
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(1).to_vec(), &[4]);
        assert_eq!(s.domain(2).to_vec(), &[4]);
    }

    #[test]
    fn cardinality_conflict() {
        let mut s = Store::new(&[vec![3], vec![3], vec![3, 4]]);
        let mut r = Registry::new();
        r.add(Box::new(Cardinality::new(vec![0, 1, 2], 3, 1)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn modulo_prunes() {
        let mut s = Store::new(&[vec![1, 3, 4, 6, 9]]);
        let mut r = Registry::new();
        r.add(Box::new(Modulo::new(0, 3, 0)));
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(0).to_vec(), &[3, 6, 9]);
    }

    #[test]
    fn modulo_conflict() {
        let mut s = Store::new(&[vec![4]]);
        let mut r = Registry::new();
        r.add(Box::new(Modulo::new(0, 3, 0)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn and_prunes() {
        let mut s = Store::new(&[vec![0, 1], vec![0, 1], vec![1]]);
        let mut r = Registry::new();
        r.add(Box::new(And::new(0, 1, 2)));
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(0).to_vec(), &[1]);
        assert_eq!(s.domain(1).to_vec(), &[1]);
    }

    #[test]
    fn and_conflict() {
        let mut s = Store::new(&[vec![0], vec![1], vec![1]]);
        let mut r = Registry::new();
        r.add(Box::new(And::new(0, 1, 2)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn or_prunes() {
        // z = x ∨ y, x = 0, z = 0 -> y must be 0
        let mut s = Store::new(&[vec![0], vec![0, 1], vec![0]]);
        let mut r = Registry::new();
        r.add(Box::new(Or::new(0, 1, 2)));
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(1).to_vec(), &[0]);
    }

    #[test]
    fn xor_prunes() {
        let mut s = Store::new(&[vec![1], vec![0, 1], vec![1]]);
        let mut r = Registry::new();
        r.add(Box::new(Xor::new(0, 1, 2)));
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(1).to_vec(), &[0]);
    }

    #[test]
    fn element_conflict_on_out_of_range_idx() {
        let mut s = Store::new(&[vec![1], vec![2], vec![3], vec![3], vec![1]]);
        let mut r = Registry::new();
        r.add(Box::new(Element::new(vec![0, 1, 2], 3, 4)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }

    #[test]
    fn modulo_conflict_on_zero_divisor() {
        let mut s = Store::new(&[vec![1, 2]]);
        let mut r = Registry::new();
        r.add(Box::new(Modulo::new(0, 0, 0)));
        assert_eq!(r.fixpoint(&mut s), Err(Conflict));
    }
}
