use crate::propagators::{AllDiff, LinearEq, NeqLinear, NonZero, Propagator, Registry};
use crate::store::{Conflict, Store};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug)]
pub enum Constraint {
    NeqLinear(Vec<(usize, i64)>, i64),
    AllDiff(Vec<usize>),
    LinearEq(Vec<(usize, i64)>, i64),
    NonZero(usize),
}

fn to_propagator(c: &Constraint) -> Box<dyn Propagator> {
    match c {
        Constraint::NeqLinear(t, k) => Box::new(NeqLinear::new(t.clone(), *k)),
        Constraint::AllDiff(v) => Box::new(AllDiff::new(v.clone())),
        Constraint::LinearEq(t, r) => Box::new(LinearEq::new(t.clone(), *r)),
        Constraint::NonZero(i) => Box::new(NonZero::new(*i)),
    }
}

pub trait Solver {
    fn post(&mut self, c: Constraint) -> Result<(), Conflict>;
    fn entails(&self, c: &Constraint) -> bool;
    fn solve(&mut self) -> Option<Vec<i64>>;
    fn commit(&mut self, cell: usize, value: i64) -> Result<(), Conflict>;
    fn is_fixed(&self, cell: usize) -> bool;
    fn value(&self, cell: usize) -> i64;
}

pub struct CustomSolver {
    domains: Vec<Vec<i64>>,
    constraints: Vec<Constraint>,
    store: Store,
    registry: Registry,
    cancel: Arc<AtomicBool>,
}

impl CustomSolver {
    pub fn new(domains: &[Vec<i64>]) -> CustomSolver {
        CustomSolver {
            domains: domains.to_vec(),
            constraints: Vec::new(),
            store: Store::new(domains),
            registry: Registry::new(),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Solver for CustomSolver {
    fn post(&mut self, c: Constraint) -> Result<(), Conflict> {
        self.constraints.push(c.clone());
        self.registry.add(to_propagator(&c));
        self.registry.fixpoint(&mut self.store)?;
        Ok(())
    }

    fn entails(&self, c: &Constraint) -> bool {
        // ponytail: entailment = all involved cells fixed and constraint holds;
        // add real entailment (bounds reasoning) when needed.
        let involved: Vec<usize> = match c {
            Constraint::NeqLinear(t, _) | Constraint::LinearEq(t, _) => t.iter().map(|x| x.0).collect(),
            Constraint::AllDiff(v) => v.clone(),
            Constraint::NonZero(i) => vec![*i],
        };
        if !involved.iter().all(|&c| self.store.is_fixed(c)) { return false; }
        let mut scratch = Store::new(&self.domains);
        for &c in &involved { scratch.commit(c, self.store.value(c)).unwrap(); }
        to_propagator(c).propagate(&mut scratch).is_ok()
    }

    fn solve(&mut self) -> Option<Vec<i64>> {
        self.dfs()
    }

    fn commit(&mut self, cell: usize, value: i64) -> Result<(), Conflict> {
        self.store.commit(cell, value)?;
        self.registry.fixpoint(&mut self.store)?;
        Ok(())
    }

    fn is_fixed(&self, cell: usize) -> bool { self.store.is_fixed(cell) }
    fn value(&self, cell: usize) -> i64 { self.store.value(cell) }
}

impl CustomSolver {
    fn dfs(&mut self) -> Option<Vec<i64>> {
        if self.cancel.load(Ordering::Relaxed) { return None; }
        let cell = match self.store.first_open() {
            Some(c) => c,
            None => return Some(self.store.assignment()),
        };
        let domain: Vec<i64> = self.store.domain(cell).to_vec();
        for &v in &domain {
            let m = self.store.mark();
            if self.store.commit(cell, v).is_err() { continue; }
            if self.registry.fixpoint(&mut self.store).is_err() {
                self.store.undo_to(m);
                continue;
            }
            if let Some(sol) = self.dfs() { return Some(sol); }
            self.store.undo_to(m);
        }
        None
    }

    pub fn solve_parallel(&mut self) -> Option<Vec<i64>> {
        let cell = self.store.first_open()?;
        let domain: Vec<i64> = self.store.domain(cell).to_vec();
        let (first, rest) = (domain[0], domain[1..].to_vec());

        // main branch on the calling thread
        let m = self.store.mark();
        if self.store.commit(cell, first).is_ok() {
            if self.registry.fixpoint(&mut self.store).is_ok() {
                if let Some(sol) = self.dfs() { return Some(sol); }
            }
        }
        self.store.undo_to(m);

        // remaining splits on rayon workers, first solution wins
        let flag = self.cancel.clone();
        let result: Arc<Mutex<Option<Vec<i64>>>> = Arc::new(Mutex::new(None));
        let constraints = self.constraints.clone();
        let domains = self.domains.clone();
        rayon::scope(|scope| {
            for &v in &rest {
                let (c, d, r, f) = (cell, domains.clone(), result.clone(), flag.clone());
                let cons = constraints.clone();
                scope.spawn(move |_| {
                    if f.load(Ordering::Relaxed) { return; }
                    let mut sub = CustomSolver::new(&d);
                    sub.cancel = f;
                    for k in cons { sub.post(k).unwrap(); }
                    if sub.store.commit(c, v).is_ok() && sub.registry.fixpoint(&mut sub.store).is_ok() {
                        if let Some(sol) = sub.dfs() {
                            let mut slot = r.lock().unwrap();
                            if slot.is_none() { *slot = Some(sol); }
                            sub.cancel.store(true, Ordering::Relaxed);
                        }
                    }
                });
            }
        });
        Arc::try_unwrap(result).ok().and_then(|m| m.into_inner().ok()).flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neq(i: usize, j: usize, k: i64) -> Constraint {
        Constraint::NeqLinear(vec![(i, 1), (j, -1)], k)
    }

    #[test]
    fn eight_queens() {
        let domains: Vec<Vec<i64>> = (0..8).map(|_| (0..8).collect()).collect();
        let mut s = CustomSolver::new(&domains);
        for i in 0..8 {
            for j in (i + 1)..8 {
                s.post(neq(i, j, 0)).unwrap();            // column
                s.post(neq(i, j, i as i64 - j as i64)).unwrap();  // main diagonal
                s.post(neq(i, j, j as i64 - i as i64)).unwrap();  // anti-diagonal
            }
        }
        let sol = s.solve().unwrap();
        for row in 0..8 {
            for col in (row + 1)..8 {
                assert_ne!(sol[row], sol[col]);
                assert_ne!(sol[row] - row as i64, sol[col] - col as i64);
                assert_ne!(sol[row] + row as i64, sol[col] + col as i64);
            }
        }
    }

    #[test]
    fn send_more_money() {
        // S=0 E=1 N=2 D=3 M=4 O=5 R=6 Y=7
        let domains: Vec<Vec<i64>> = vec![(0..10).collect(); 8];
        let mut s = CustomSolver::new(&domains);
        s.post(Constraint::AllDiff((0..8).collect())).unwrap();
        s.post(Constraint::NonZero(0)).unwrap(); // S
        s.post(Constraint::NonZero(4)).unwrap(); // M
        s.post(Constraint::LinearEq(
            vec![(0, 1000), (1, 100), (2, 10), (3, 1),
                 (4, 1000), (5, 100), (6, 10), (1, 1),
                 (4, -10000), (5, -1000), (2, -100), (1, -10), (7, -1)],
            0,
        )).unwrap();
        assert_eq!(s.solve().unwrap(), vec![9, 5, 6, 7, 1, 0, 8, 2]);
    }

    #[test]
    fn parallel_twelve_queens_is_valid() {
        let domains: Vec<Vec<i64>> = (0..12).map(|_| (0..12).collect()).collect();
        let mut s = CustomSolver::new(&domains);
        for i in 0..12 {
            for j in (i + 1)..12 {
                s.post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], 0)).unwrap();
                s.post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], i as i64 - j as i64)).unwrap();
                s.post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], j as i64 - i as i64)).unwrap();
            }
        }
        let sol = s.solve_parallel().unwrap();
        assert_eq!(sol.len(), 12);
        for row in 0..12 {
            for col in (row + 1)..12 {
                assert_ne!(sol[row], sol[col]);
                assert_ne!(sol[row] - row as i64, sol[col] - col as i64);
                assert_ne!(sol[row] + row as i64, sol[col] + col as i64);
            }
        }
    }

    #[test]
    fn parallel_beats_sequential_14_queens() {
        // 14-Queens plus a sum constraint satisfiable only with row0 >= 4, so the
        // first split value (row0 = 0) is an unsatisfiable subtree that sequential
        // DFS must exhaust before reaching a solution, while rayon workers cover
        // the remaining splits concurrently.
        let domains: Vec<Vec<i64>> = (0..14).map(|_| (0..14).collect()).collect();
        let build = || {
            let mut s = CustomSolver::new(&domains);
            for i in 0..14 {
                for j in (i + 1)..14 {
                    s.post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], 0)).unwrap();
                    s.post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], i as i64 - j as i64)).unwrap();
                    s.post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], j as i64 - i as i64)).unwrap();
                }
            }
            s.post(Constraint::LinearEq(vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 1)], 50)).unwrap();
            s
        };
        let valid = |sol: &Vec<i64>| {
            for row in 0..14 {
                for col in (row + 1)..14 {
                    if sol[row] == sol[col]
                        || sol[row] - row as i64 == sol[col] - col as i64
                        || sol[row] + row as i64 == sol[col] + col as i64 { return false; }
                }
            }
            true
        };
        let t0 = std::time::Instant::now();
        let seq = build().solve().unwrap();
        let sd = t0.elapsed();
        let t1 = std::time::Instant::now();
        let par = build().solve_parallel().unwrap();
        let pd = t1.elapsed();
        assert!(valid(&seq) && valid(&par));
        assert!(pd < sd, "parallel {pd:?} >= sequential {sd:?}");
    }
}
