#[derive(Debug, PartialEq)]
pub struct Conflict;

pub struct Store {
    cells: Vec<Vec<i64>>,
    trail: Vec<(usize, i64)>,
}

impl Store {
    pub fn new(domains: &[Vec<i64>]) -> Store {
        Store { cells: domains.to_vec(), trail: Vec::new() }
    }

    pub fn domain(&self, cell: usize) -> &[i64] { &self.cells[cell] }
    pub fn is_fixed(&self, cell: usize) -> bool { self.cells[cell].len() == 1 }
    pub fn value(&self, cell: usize) -> i64 { self.cells[cell][0] }
    pub fn first_open(&self) -> Option<usize> {
        (0..self.cells.len()).find(|&c| !self.is_fixed(c))
    }
    pub fn mark(&self) -> usize { self.trail.len() }
    pub fn trail_len(&self) -> usize { self.trail.len() }

    pub fn remove(&mut self, cell: usize, value: i64) -> Result<(), Conflict> {
        let d = &mut self.cells[cell];
        if let Some(pos) = d.iter().position(|&v| v == value) {
            d.remove(pos);
            self.trail.push((cell, value));
            if d.is_empty() { return Err(Conflict); }
        }
        Ok(())
    }

    pub fn commit(&mut self, cell: usize, value: i64) -> Result<(), Conflict> {
        if !self.cells[cell].contains(&value) { return Err(Conflict); }
        for v in self.cells[cell].clone() {
            if v != value { self.remove(cell, v)?; }
        }
        Ok(())
    }

    pub fn undo_to(&mut self, mark: usize) {
        for (cell, value) in self.trail.drain(mark..).rev() {
            self.cells[cell].push(value);
            self.cells[cell].sort();
        }
    }

    pub fn assignment(&self) -> Vec<i64> {
        self.cells.iter().map(|d| d[0]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_and_undo() {
        let mut s = Store::new(&[vec![1, 2, 3]]);
        s.remove(0, 2).unwrap();
        assert_eq!(s.domain(0), &[1, 3]);
        let m = s.mark();
        s.remove(0, 1).unwrap();
        s.undo_to(m);
        assert_eq!(s.domain(0), &[1, 3]);
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
