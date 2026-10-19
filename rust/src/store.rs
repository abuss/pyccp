/// Threshold for auto-converting large Vec domains to Bounds representation
const BOUNDS_CONVERSION_THRESHOLD: usize = 1000;

/// Bounds tracking for propagation reasoning (min/max per variable)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: i64,
    pub max: i64,
}

impl Bounds {
    /// Create new bounds
    pub fn new(min: i64, max: i64) -> Self {
        Bounds { min, max }
    }

    /// Check if value is within bounds
    pub fn contains(&self, value: i64) -> bool {
        value >= self.min && value <= self.max
    }

    /// Intersect two bounds. Returns None if intersection is empty.
    pub fn intersect(&self, other: &Bounds) -> Option<Bounds> {
        let new_min = self.min.max(other.min);
        let new_max = self.max.min(other.max);
        if new_min <= new_max {
            Some(Bounds::new(new_min, new_max))
        } else {
            None
        }
    }
}

/// Hybrid domain representation: Vec for small domains, Bounds for large ones
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    Vec(std::vec::Vec<i64>),
    Bounds(i64, i64),
}

impl Domain {
    /// Create a new domain from a vector of values
    pub fn new(mut values: std::vec::Vec<i64>) -> Self {
        values.sort();
        values.dedup();
        
        // Auto-convert large domains to Bounds representation
        if values.len() > BOUNDS_CONVERSION_THRESHOLD {
            if let (Some(&min), Some(&max)) = (values.first(), values.last()) {
                return Domain::Bounds(min, max);
            }
        }
        
        Domain::Vec(values)
    }

    /// Check if domain is fixed (single value)
    pub fn is_fixed(&self) -> bool {
        match self {
            Domain::Vec(vec) => vec.len() == 1,
            Domain::Bounds(min, max) => min == max,
        }
    }

    /// Get the fixed value. Panics if not fixed.
    pub fn value(&self) -> i64 {
        match self {
            Domain::Vec(vec) => {
                assert_eq!(vec.len(), 1, "value() called on non-fixed Vec domain");
                vec[0]
            }
            Domain::Bounds(min, max) => {
                assert_eq!(min, max, "value() called on non-fixed Bounds domain");
                *min
            }
        }
    }

    /// Get bounds of the domain
    pub fn bounds(&self) -> Bounds {
        match self {
            Domain::Vec(vec) => {
                let min = vec.first().copied().unwrap_or(0);
                let max = vec.last().copied().unwrap_or(0);
                Bounds::new(min, max)
            }
            Domain::Bounds(min, max) => Bounds::new(*min, *max),
        }
    }

    /// Remove a value from the domain. Returns true if value was present.
    /// For Bounds, only boundary values can be removed.
    pub fn remove(&mut self, value: i64) -> bool {
        match self {
            Domain::Vec(ref mut vec) => {
                // Domains are kept sorted, so binary_search beats a linear scan.
                match vec.binary_search(&value) {
                    Ok(pos) => {
                        vec.remove(pos);
                        true
                    }
                    Err(_) => false,
                }
            }
            Domain::Bounds(ref mut min, ref mut max) => {
                if value == *min && value < *max {
                    *min += 1;
                    true
                } else if value == *max && value > *min {
                    *max -= 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Tighten bounds to [new_min, new_max]. Returns Err(Conflict) if empty.
    pub fn tighten_bounds(&mut self, new_min: i64, new_max: i64) -> Result<(), Conflict> {
        if new_min > new_max {
            return Err(Conflict);
        }

        match self {
            Domain::Vec(ref mut vec) => {
                // For Vec domains, remove all values outside [new_min, new_max]
                vec.retain(|&v| v >= new_min && v <= new_max);
                if vec.is_empty() {
                    Err(Conflict)
                } else {
                    Ok(())
                }
            }
            Domain::Bounds(ref mut min, ref mut max) => {
                let new_bounds_min = new_min.max(*min);
                let new_bounds_max = new_max.min(*max);
                if new_bounds_min <= new_bounds_max {
                    *min = new_bounds_min;
                    *max = new_bounds_max;
                    Ok(())
                } else {
                    Err(Conflict)
                }
            }
        }
    }

    /// Check if the domain is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Domain::Vec(vec) => vec.is_empty(),
            Domain::Bounds(min, max) => min > max,
        }
    }

    /// Get the number of values in the domain
    pub fn len(&self) -> usize {
        match self {
            Domain::Vec(vec) => vec.len(),
            Domain::Bounds(min, max) => ((max - min) + 1).max(0) as usize,
        }
    }

    /// Check if the domain contains a value (support both &i64 and i64)
    pub fn contains(&self, value: i64) -> bool {
        match self {
            Domain::Vec(vec) => vec.binary_search(&value).is_ok(),
            Domain::Bounds(min, max) => value >= *min && value <= *max,
        }
    }

    /// Get an iterator over the domain values
    pub fn iter(&self) -> Box<dyn Iterator<Item = &i64> + '_> {
        match self {
            Domain::Vec(vec) => Box::new(vec.iter()),
            Domain::Bounds(_min, _max) => {
                // For Bounds, we can't return an iterator over all values
                // This is by design - Bounds domains are for large ranges
                Box::new(std::iter::empty())
            }
        }
    }

    /// Get the minimum value in the domain, or None if empty
    pub fn min(&self) -> Option<i64> {
        match self {
            Domain::Vec(vec) => vec.first().copied(),
            Domain::Bounds(min, max) => if min <= max { Some(*min) } else { None },
        }
    }

    /// Get the maximum value in the domain, or None if empty
    pub fn max(&self) -> Option<i64> {
        match self {
            Domain::Vec(vec) => vec.last().copied(),
            Domain::Bounds(min, max) => if min <= max { Some(*max) } else { None },
        }
    }

    /// Convert domain to a vector (for compatibility with existing code)
    pub fn to_vec(&self) -> std::vec::Vec<i64> {
        match self {
            Domain::Vec(vec) => vec.clone(),
            Domain::Bounds(min, max) => {
                if min <= max {
                    (*min..=*max).collect()
                } else {
                    vec![]
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum TrailOp {
    Remove(i64),           // Remove value from Vec domain
    TightenBounds(Bounds), // Restore old bounds for a cell
}

#[derive(Debug, PartialEq)]
pub struct Conflict;

pub struct Store {
    cells: std::vec::Vec<Domain>,
    trail: std::vec::Vec<(usize, TrailOp)>,  // Unified trail: (cell, operation)
    mrv_cache: Option<usize>,
    mrv_cache_valid: bool,
    bounds: std::vec::Vec<Bounds>,
    changed_vars: std::collections::HashSet<usize>,  // Track which variables changed
    pub objective: Option<(usize, bool)>,            // (cell_index, is_minimize)
    pub best_cost: Option<i64>,                      // best solution cost found so far
}

impl Store {
    pub fn new(domains: &[std::vec::Vec<i64>]) -> Store {
        let mut bounds = std::vec::Vec::new();
        for domain_vals in domains {
            if let Some(&min_val) = domain_vals.first() {
                if let Some(&max_val) = domain_vals.last() {
                    bounds.push(Bounds::new(min_val, max_val));
                } else {
                    bounds.push(Bounds::new(0, 0)); // Empty domain
                }
            } else {
                bounds.push(Bounds::new(0, 0)); // Empty domain
            }
        }

        Store {
            cells: domains.iter().map(|d| Domain::new(d.clone())).collect(),
            trail: std::vec::Vec::new(),
            mrv_cache: None,
            mrv_cache_valid: false,
            bounds,
            changed_vars: std::collections::HashSet::new(),
            objective: None,
            best_cost: None,
        }
    }

    pub fn domain(&self, cell: usize) -> &Domain {
        &self.cells[cell]
    }

    pub fn is_fixed(&self, cell: usize) -> bool {
        self.cells[cell].len() == 1
    }

    pub fn value(&self, cell: usize) -> i64 {
        self.cells[cell].value()
    }

    pub fn first_open(&self) -> Option<usize> {
        (0..self.cells.len()).find(|&c| !self.is_fixed(c))
    }

    pub fn mark(&self) -> usize {
        self.trail.len()
    }

    pub fn invalidate_mrv_cache(&mut self) {
        self.mrv_cache_valid = false;
    }

    pub fn mrv(&mut self) -> Option<usize> {
        if self.mrv_cache_valid {
            return self.mrv_cache;
        }
        
        let mut best_cell = None;
        let mut best_size = usize::MAX;
        
        for (cell, domain) in self.cells.iter().enumerate() {
            if !self.is_fixed(cell) {
                let size = domain.len();
                if size < best_size {
                    best_size = size;
                    best_cell = Some(cell);
                }
            }
        }
        
        self.mrv_cache = best_cell;
        self.mrv_cache_valid = true;
        best_cell
    }

    pub fn trail_len(&self) -> usize {
        self.trail.len()
    }

    pub fn remove(&mut self, cell: usize, value: i64) -> Result<(), Conflict> {
        let removed = self.cells[cell].remove(value);
        if removed {
            self.trail.push((cell, TrailOp::Remove(value)));
            self.changed_vars.insert(cell);  // Mark variable as changed
            self.invalidate_mrv_cache();
            if self.cells[cell].is_empty() {
                return Err(Conflict);
            }
        }
        Ok(())
    }

    pub fn commit(&mut self, cell: usize, value: i64) -> Result<(), Conflict> {
        if !self.cells[cell].contains(value) {
            return Err(Conflict);
        }
        let values_to_remove: std::vec::Vec<i64> = self.cells[cell]
            .iter()
            .copied()
            .filter(|&v| v != value)
            .collect();
        for v in values_to_remove {
            self.remove(cell, v)?;
        }
        self.invalidate_mrv_cache();
        Ok(())
    }

    pub fn undo_to(&mut self, mark: usize) {
        let mut affected: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for (cell, op) in self.trail.drain(mark..).rev() {
            match op {
                TrailOp::Remove(value) => {
                    if let Domain::Vec(ref mut vec) = &mut self.cells[cell] {
                        vec.push(value);
                        affected.insert(cell);
                    }
                }
                TrailOp::TightenBounds(old_bounds) => {
                    self.bounds[cell] = old_bounds;
                }
            }
        }
        // Sort once per affected cell instead of after every pushed value.
        for cell in affected {
            if let Domain::Vec(ref mut vec) = &mut self.cells[cell] {
                vec.sort();
            }
        }
        
        // Clear changed_vars when undoing to avoid stale state
        self.changed_vars.clear();
        self.invalidate_mrv_cache();
    }

    pub fn assignment(&self) -> std::vec::Vec<i64> {
        self.cells.iter().map(|d| d.min().unwrap()).collect()
    }

    pub fn cells_unfixed(&self) -> usize {
        self.cells.iter().filter(|d| d.len() > 1).count()
    }

    /// Check if a domain is represented as Bounds
    pub fn is_bounds_domain(&self, cell: usize) -> bool {
        matches!(&self.cells[cell], Domain::Bounds(_, _))
    }

    /// Check if a domain is represented as Vec
    pub fn is_vec_domain(&self, cell: usize) -> bool {
        matches!(&self.cells[cell], Domain::Vec(_))
    }

    /// Get domain as Vec, or None if it's a Bounds domain
    pub fn domain_or_none(&self, cell: usize) -> Option<std::vec::Vec<i64>> {
        match &self.cells[cell] {
            Domain::Vec(vec) => Some(vec.clone()),
            Domain::Bounds(_, _) => None,
        }
    }

    /// Tighten domain bounds and record to trail (new_min, new_max version)
    pub fn tighten_bounds(&mut self, cell: usize, new_min: i64, new_max: i64) -> Result<(), Conflict> {
        // Record old state to trail for undo
        let old_bounds = self.bounds[cell];
        self.trail.push((cell, TrailOp::TightenBounds(old_bounds)));
        
        // Tighten the domain itself
        self.cells[cell].tighten_bounds(new_min, new_max)?;
        
        // Also tighten the bounds field
        let new_bounds = Bounds::new(new_min, new_max);
        if let Some(tightened) = old_bounds.intersect(&new_bounds) {
            self.bounds[cell] = tightened;
            self.invalidate_mrv_cache();
            Ok(())
        } else {
            Err(Conflict)
        }
    }

    pub fn bounds(&self, cell: usize) -> Bounds {
        self.bounds[cell]
    }

    pub fn set_bounds(&mut self, cell: usize, b: Bounds) -> Result<(), Conflict> {
        self.bounds[cell] = b;
        self.invalidate_mrv_cache();
        Ok(())
    }

    /// Tighten bounds with a Bounds object (delegates to the new version)
    pub fn tighten_bounds_with_bounds(&mut self, cell: usize, new_bounds: Bounds) -> Result<(), Conflict> {
        self.tighten_bounds(cell, new_bounds.min, new_bounds.max)
    }

    pub fn get_changed_vars(&self) -> std::vec::Vec<usize> {
        self.changed_vars.iter().copied().collect()
    }

    pub fn clear_changed_vars(&mut self) {
        self.changed_vars.clear();
    }

    pub fn mark_var_changed(&mut self, var: usize) {
        self.changed_vars.insert(var);
    }

    pub fn num_variables(&self) -> usize {
        self.cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_and_undo() {
        let mut s = Store::new(&[vec![1, 2, 3]]);
        s.remove(0, 2).unwrap();
        let domain_values: std::vec::Vec<i64> = s.domain(0).iter().copied().collect();
        assert_eq!(domain_values, vec![1, 3]);
        let m = s.mark();
        s.remove(0, 1).unwrap();
        s.undo_to(m);
        let domain_values: std::vec::Vec<i64> = s.domain(0).iter().copied().collect();
        assert_eq!(domain_values, vec![1, 3]);
    }

    #[test]
    fn empty_domain_is_conflict() {
        let mut s = Store::new(&[vec![1]]);
        assert_eq!(s.remove(0, 1), Err(Conflict));
    }

    #[test]
    fn commit_fixes_cell() {
        let mut s = Store::new(&[vec![1, 2, 3]]);
        s.commit(0, 2).unwrap();
        assert!(s.is_fixed(0));
        assert_eq!(s.value(0), 2);
        assert_eq!(s.commit(0, 2), Ok(()));   // idempotent
        assert_eq!(s.commit(0, 1), Err(Conflict));
    }
}
