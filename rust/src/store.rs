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

/// Hybrid domain representation: Vec for small domains, BitSet for large ones
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    Vec(std::vec::Vec<i64>),
    // BitSet variant reserved for future optimization
}

impl Domain {
    /// Create a new domain from a vector of values
    pub fn new(mut values: std::vec::Vec<i64>) -> Self {
        values.sort();
        values.dedup();
        Domain::Vec(values)
    }

    /// Remove a value from the domain. Returns true if value was present.
    pub fn remove(&mut self, value: i64) -> bool {
        match self {
            Domain::Vec(ref mut vec) => {
                if let Some(pos) = vec.iter().position(|&v| v == value) {
                    vec.remove(pos);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Check if the domain is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Domain::Vec(vec) => vec.is_empty(),
        }
    }

    /// Get the number of values in the domain
    pub fn len(&self) -> usize {
        match self {
            Domain::Vec(vec) => vec.len(),
        }
    }

    /// Check if the domain contains a value (support both &i64 and i64)
    pub fn contains(&self, value: i64) -> bool {
        match self {
            Domain::Vec(vec) => vec.binary_search(&value).is_ok(),
        }
    }

    /// Get an iterator over the domain values
    pub fn iter(&self) -> Box<dyn Iterator<Item = &i64> + '_> {
        match self {
            Domain::Vec(vec) => Box::new(vec.iter()),
        }
    }

    /// Get the minimum value in the domain, or None if empty
    pub fn min(&self) -> Option<i64> {
        match self {
            Domain::Vec(vec) => vec.first().copied(),
        }
    }

    /// Get the maximum value in the domain, or None if empty
    pub fn max(&self) -> Option<i64> {
        match self {
            Domain::Vec(vec) => vec.last().copied(),
        }
    }

    /// Convert domain to a vector (for compatibility with existing code)
    pub fn to_vec(&self) -> std::vec::Vec<i64> {
        match self {
            Domain::Vec(vec) => vec.clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Conflict;

pub struct Store {
    cells: std::vec::Vec<Domain>,
    trail: std::vec::Vec<(usize, i64)>,
    mrv_cache: Option<usize>,
    mrv_cache_valid: bool,
    bounds: std::vec::Vec<Bounds>,
}

impl Store {
    pub fn new(domains: &[std::vec::Vec<i64>]) -> Store {
        let mut bounds = std::vec::Vec::new();
        for domain_vals in domains {
            if let Some(&min_val) = domain_vals.first() {
                if let Some(&max_val) = domain_vals.last() {
                    bounds.push(Bounds::new(min_val, max_val));
                }
            }
        }

        Store {
            cells: domains.iter().map(|d| Domain::new(d.clone())).collect(),
            trail: std::vec::Vec::new(),
            mrv_cache: None,
            mrv_cache_valid: false,
            bounds,
        }
    }

    pub fn domain(&self, cell: usize) -> &Domain {
        &self.cells[cell]
    }

    pub fn is_fixed(&self, cell: usize) -> bool {
        self.cells[cell].len() == 1
    }

    pub fn value(&self, cell: usize) -> i64 {
        match &self.cells[cell] {
            Domain::Vec(vec) => vec[0],
        }
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
            self.trail.push((cell, value));
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
        for (cell, value) in self.trail.drain(mark..).rev() {
            // For Vec domains, we need to insert in sorted order
            match &mut self.cells[cell] {
                Domain::Vec(ref mut vec) => {
                    vec.push(value);
                    vec.sort();
                }
            }
        }
        self.invalidate_mrv_cache();
    }

    pub fn assignment(&self) -> std::vec::Vec<i64> {
        self.cells.iter().map(|d| d.min().unwrap()).collect()
    }

    pub fn cells_unfixed(&self) -> usize {
        self.cells.iter().filter(|d| d.len() > 1).count()
    }

    pub fn bounds(&self, cell: usize) -> Bounds {
        self.bounds[cell]
    }

    pub fn set_bounds(&mut self, cell: usize, b: Bounds) -> Result<(), Conflict> {
        self.bounds[cell] = b;
        self.invalidate_mrv_cache();
        Ok(())
    }

    pub fn tighten_bounds(&mut self, cell: usize, new_bounds: Bounds) -> Result<(), Conflict> {
        let current = self.bounds[cell];
        if let Some(tightened) = current.intersect(&new_bounds) {
            self.bounds[cell] = tightened;
            self.invalidate_mrv_cache();
            Ok(())
        } else {
            Err(Conflict)
        }
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
