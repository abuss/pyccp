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
}
impl Propagator for NeqLinear {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // ponytail: prunes only when all-but-one term is fixed; add bounds
        // reasoning when interval propagation is needed.
        let open: Vec<(usize, i64)> = self.terms.iter()
            .filter(|(c, _)| !store.is_fixed(*c)).copied().collect();
        let mut sum = 0;
        for (c, k) in &self.terms {
            if store.is_fixed(*c) { sum += k * store.value(*c); }
        }
        if open.is_empty() {
            return if sum == self.k { Err(Conflict) } else { Ok(()) };
        }
        if open.len() == 1 {
            let (c, coef) = open[0];
            let rem = self.k - sum;
            if rem % coef == 0 {
                let v = rem / coef;
                if store.domain(c).contains(&v) { store.remove(c, v)?; }
            }
        }
        Ok(())
    }
}

pub struct AllDiff { cells: Vec<usize> }
impl AllDiff {
    pub fn new(cells: Vec<usize>) -> AllDiff { AllDiff { cells } }
}
impl Propagator for AllDiff {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        for &i in &self.cells {
            if store.is_fixed(i) {
                let v = store.value(i);
                for &j in &self.cells {
                    if j != i && store.domain(j).contains(&v) { store.remove(j, v)?; }
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
}
impl Propagator for LinearEq {
    fn cells(&self) -> &[usize] { &self.cells }
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict> {
        // ponytail: propagates only when all-but-one term is fixed; full
        // interval arithmetic when bounds-based pruning is needed.
        let open: Vec<(usize, i64)> = self.terms.iter()
            .filter(|(c, _)| !store.is_fixed(*c)).copied().collect();
        if open.len() > 1 { return Ok(()); }
        let mut sum = self.rhs;
        for (c, k) in &self.terms {
            if store.is_fixed(*c) { sum -= k * store.value(*c); }
        }
        if open.len() == 1 {
            let (c, k) = open[0];
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

pub struct Registry { props: Vec<Box<dyn Propagator>> }
impl Registry {
    pub fn new() -> Registry { Registry { props: Vec::new() } }
    pub fn add(&mut self, p: Box<dyn Propagator>) { self.props.push(p); }

    pub fn fixpoint(&mut self, store: &mut Store) -> Result<(), Conflict> {
        loop {
            let before = store.trail_len();
            for p in &mut self.props { p.propagate(store)?; }
            if store.trail_len() == before { return Ok(()); }
        }
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
        assert_eq!(s.domain(0), &[2]);
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
        assert_eq!(s.domain(1), &[1]);
    }

    #[test]
    fn all_diff_prunes() {
        let mut s = Store::new(&[vec![1, 2], vec![1, 2], vec![1, 2, 3]]);
        let mut r = Registry::new();
        r.add(Box::new(AllDiff::new(vec![0, 1, 2])));
        s.commit(0, 1).unwrap();
        s.commit(1, 2).unwrap();
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(2), &[3]);
    }

    #[test]
    fn linear_eq_propagates() {
        // y - x == 1, x fixed to 1 -> y fixed to 2
        let mut s = Store::new(&[vec![1, 2], vec![2, 3]]);
        let mut r = Registry::new();
        r.add(Box::new(LinearEq::new(vec![(1, 1), (0, -1)], 1)));
        s.commit(0, 1).unwrap();
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(1), &[2]);
    }

    #[test]
    fn nonzero_prunes() {
        let mut s = Store::new(&[vec![0, 1, 2]]);
        let mut r = Registry::new();
        r.add(Box::new(NonZero::new(0)));
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(0), &[1, 2]);
    }
}
