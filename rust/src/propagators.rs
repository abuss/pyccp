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
            if coef == 0 { return Err(Conflict); }
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
        // ponytail: prunes only when one side is fixed; bounds reasoning later.
        let [i, j] = self.cells;
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
        // ponytail: prunes only when one side is fixed.
        let [i, j] = self.cells;
        if store.is_fixed(i) && store.is_fixed(j) {
            return if store.value(i) * store.value(j) == self.k { Ok(()) } else { Err(Conflict) };
        }
        for (a, b) in [(i, j), (j, i)] {
            if store.is_fixed(a) {
                let v = store.value(a);
                if v == 0 { if self.k != 0 { return Err(Conflict); } continue; }
                if self.k % v != 0 { return Err(Conflict); }
                store.commit(b, self.k / v)?;
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
        // ponytail: prunes only when idx or y is fixed; full bidirectional
        // domain intersection when needed.
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
        }
        if store.is_fixed(self.y) {
            let yv = store.value(self.y);
            for c in self.array.clone() {
                if store.is_fixed(c) {
                    if store.value(c) != yv { store.remove(self.idx, c as i64)?; }
                    else { store.commit(self.idx, c as i64)?; }
                }
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
        assert_eq!(s.domain(0), &[1, 2]);
    }

    #[test]
    fn less_than_prunes() {
        let mut s = Store::new(&[vec![1, 2, 3], vec![1, 2, 3]]);
        let mut r = Registry::new();
        r.add(Box::new(LessThan::new(0, 1)));
        s.commit(1, 2).unwrap();
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(0), &[1]);
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
        assert_eq!(s.domain(1), &[6]);
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
        assert_eq!(s.domain(4), &[1]);
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
        assert_eq!(s.domain(1), &[4]);
        assert_eq!(s.domain(2), &[4]);
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
        assert_eq!(s.domain(0), &[3, 6, 9]);
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
        assert_eq!(s.domain(0), &[1]);
        assert_eq!(s.domain(1), &[1]);
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
        assert_eq!(s.domain(1), &[0]);
    }

    #[test]
    fn xor_prunes() {
        let mut s = Store::new(&[vec![1], vec![0, 1], vec![1]]);
        let mut r = Registry::new();
        r.add(Box::new(Xor::new(0, 1, 2)));
        r.fixpoint(&mut s).unwrap();
        assert_eq!(s.domain(1), &[0]);
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
