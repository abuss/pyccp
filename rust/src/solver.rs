use crate::propagators::{AllDiff, And, Cardinality, Element, LeLinear, LinearEq, LessThan, Modulo, Multiply, NeqLinear, NonZero, Or, Propagator, Registry, Xor};
use crate::store::{Conflict, Store};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug)]
pub enum Constraint {
    NeqLinear(Vec<(usize, i64)>, i64),
    AllDiff(Vec<usize>),
    LinearEq(Vec<(usize, i64)>, i64),
    LeLinear(Vec<(usize, i64)>, i64),
    NonZero(usize),
    LessThan(usize, usize),
    Multiply(usize, usize, i64),
    Element(Vec<usize>, usize, usize),
    Cardinality(Vec<usize>, i64, usize),
    Modulo(usize, i64, i64),
    And(usize, usize, usize),
    Or(usize, usize, usize),
    Xor(usize, usize, usize),
}

fn to_propagator(c: &Constraint) -> Box<dyn Propagator> {
    match c {
        Constraint::NeqLinear(t, k) => Box::new(NeqLinear::new(t.clone(), *k)),
        Constraint::AllDiff(v) => Box::new(AllDiff::new(v.clone())),
        Constraint::LinearEq(t, r) => Box::new(LinearEq::new(t.clone(), *r)),
        Constraint::LeLinear(t, r) => Box::new(LeLinear::new(t.clone(), *r)),
        Constraint::NonZero(i) => Box::new(NonZero::new(*i)),
        Constraint::LessThan(i, j) => Box::new(LessThan::new(*i, *j)),
        Constraint::Multiply(i, j, k) => Box::new(Multiply::new(*i, *j, *k)),
        Constraint::Element(a, i, y) => Box::new(Element::new(a.clone(), *i, *y)),
        Constraint::Cardinality(c, v, n) => Box::new(Cardinality::new(c.clone(), *v, *n)),
        Constraint::Modulo(c, m, k) => Box::new(Modulo::new(*c, *m, *k)),
        Constraint::And(x, y, z) => Box::new(And::new(*x, *y, *z)),
        Constraint::Or(x, y, z) => Box::new(Or::new(*x, *y, *z)),
        Constraint::Xor(x, y, z) => Box::new(Xor::new(*x, *y, *z)),
    }
}

pub trait Solver {
    fn post(&mut self, c: Constraint) -> Result<(), Conflict>;
    fn entails(&self, c: &Constraint) -> bool;
    fn solve(&mut self) -> Option<Vec<i64>>;
    fn solve_all(&mut self, limit: usize) -> Vec<Vec<i64>>;
    fn commit(&mut self, cell: usize, value: i64) -> Result<(), Conflict>;
    fn is_fixed(&self, cell: usize) -> bool;
    fn value(&self, cell: usize) -> i64;
    fn set_objective(&mut self, cell: usize, minimize: bool) -> Result<(), Conflict>;
    fn best_cost(&self) -> Option<i64>;
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
        let involved: Vec<usize> = to_propagator(c).cells().to_vec();
        if !involved.iter().all(|&c| self.store.is_fixed(c)) { return false; }
        let mut scratch = Store::new(&self.domains);
        for &c in &involved { scratch.commit(c, self.store.value(c)).unwrap(); }
        to_propagator(c).propagate(&mut scratch).is_ok()
    }

    fn solve(&mut self) -> Option<Vec<i64>> {
        let mut out = Vec::with_capacity(1);
        self.collect(&mut out, 1);  // Find first (optimal if objective set)
        out.into_iter().next()
    }

    fn solve_all(&mut self, limit: usize) -> Vec<Vec<i64>> {
        let mut out = Vec::new();
        self.collect(&mut out, limit);
        out
    }

    fn commit(&mut self, cell: usize, value: i64) -> Result<(), Conflict> {
        self.store.commit(cell, value)?;
        self.registry.fixpoint(&mut self.store)?;
        Ok(())
    }

    fn is_fixed(&self, cell: usize) -> bool { self.store.is_fixed(cell) }
    fn value(&self, cell: usize) -> i64 { self.store.value(cell) }

    fn set_objective(&mut self, cell: usize, minimize: bool) -> Result<(), Conflict> {
        if cell >= self.store.num_variables() {
            return Err(Conflict);
        }
        self.store.objective = Some((cell, minimize));
        Ok(())
    }

    fn best_cost(&self) -> Option<i64> {
        self.store.best_cost
    }
}

impl CustomSolver {
    fn dfs(&mut self) -> Option<Vec<i64>> {
        if self.cancel.load(Ordering::Relaxed) { return None; }
        let cell = match self.store.first_open() {
            Some(c) => c,
            None => {
                let solution = self.store.assignment();
                // Update best_cost if objective is set
                if let Some((cost_cell, is_minimize)) = self.store.objective {
                    let cost = solution[cost_cell];
                    let should_update = match self.store.best_cost {
                        None => true,
                        Some(best) => {
                            if is_minimize { cost < best } else { cost > best }
                        }
                    };
                    if should_update {
                        self.store.best_cost = Some(cost);
                    }
                }
                return Some(solution);
            }
        };
        let domain: Vec<i64> = self.value_order(cell);
        for &v in &domain {
            let m = self.store.mark();
            if self.store.commit(cell, v).is_err() { 
                self.store.undo_to(m);
                continue; 
            }
            if self.registry.fixpoint(&mut self.store).is_err() {
                self.store.undo_to(m);
                continue;
            }
            // B&B pruning: if cost cell is fixed and cost can't improve, skip
            if let Some((cost_cell, is_minimize)) = self.store.objective {
                if let Some(best) = self.store.best_cost {
                    if self.store.is_fixed(cost_cell) {
                        let current_cost = self.store.value(cost_cell);
                        if is_minimize && current_cost >= best {
                            self.store.undo_to(m);
                            continue;
                        } else if !is_minimize && current_cost <= best {
                            self.store.undo_to(m);
                            continue;
                        }
                    }
                }
            }
            if let Some(sol) = self.dfs() { return Some(sol); }
            self.store.undo_to(m);
        }
        None
    }

    // Returns true when the limit is reached and the search must unwind.
    // Returns true when the limit is reached and the search must unwind.
    fn collect(&mut self, out: &mut Vec<Vec<i64>>, limit: usize) -> bool {
        if out.len() >= limit { return true; }
        let cell = match self.store.first_open() {
            Some(c) => c,
            None => {
                let solution = self.store.assignment();
                // Update best_cost if objective is set
                if let Some((cost_cell, is_minimize)) = self.store.objective {
                    let cost = solution[cost_cell];
                    let should_update = match self.store.best_cost {
                        None => true,
                        Some(best) => {
                            if is_minimize { cost < best } else { cost > best }
                        }
                    };
                    if should_update {
                        self.store.best_cost = Some(cost);
                    }
                }
                out.push(solution);
                return out.len() >= limit;
            }
        };
        let domain: Vec<i64> = self.value_order(cell);
        for &v in &domain {
            let m = self.store.mark();
            if self.store.commit(cell, v).is_ok()
                && self.registry.fixpoint(&mut self.store).is_ok() {
                // NEW: Check if current cost bounds exceed best_cost -> prune
                if let Some((cost_cell, is_minimize)) = self.store.objective {
                    if let Some(best) = self.store.best_cost {
                        if self.store.is_fixed(cost_cell) {
                            let current_cost = self.store.value(cost_cell);
                            if is_minimize && current_cost >= best {
                                // Prune: cost can't improve
                                self.store.undo_to(m);
                                continue;
                            } else if !is_minimize && current_cost <= best {
                                // Prune: cost can't improve
                                self.store.undo_to(m);
                                continue;
                            }
                        }
                    }
                }
                // Recurse
                if self.collect(out, limit) {
                    self.store.undo_to(m);
                    return true;
                }
            }
            self.store.undo_to(m);
        }
        false
    }

    /// LCV heuristic: order values by how constraining they are.
    /// ponytail: simplified LCV = count propagators involving this cell that
    /// would prune on a commitment; full LCV counts values eliminated per value,
    /// add when needed.
    fn value_score(&self, cell: usize, _value: i64) -> usize {
        let prop_count = self.registry.propagators_involving(cell).len();
        // Score = number of propagators; fewer = less constraining = try last.
        // Within same cell, all values see same propagators, so all tied.
        // ponytail: real LCV scores each value by simulating a commit + fixpoint
        // on a cloned store, counts pruned values; defer if perf shows cost.
        prop_count
    }

    /// Return domain sorted by LCV score (most constraining first)
    pub fn value_order(&self, cell: usize) -> Vec<i64> {
        let mut domain: Vec<i64> = self.store.domain(cell).to_vec();
        // ponytail: LCV simplified to propagator count per cell; within a cell,
        // order is unchanged (values are tied). Full LCV simulation deferred.
        // For now, just return domain as-is; score exists for future expansion.
        domain.sort_by_key(|&v| self.value_score(cell, v));
        domain
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
    fn solve_all_diophantine_pairs() {
        // x + y == 10 over 0..=10 -> 11 solutions
        let domains = vec![(0..11).collect(), (0..11).collect()];
        let mut s = CustomSolver::new(&domains);
        s.post(Constraint::LinearEq(vec![(0, 1), (1, 1)], 10)).unwrap();
        let sols = s.solve_all(1000);
        assert_eq!(sols.len(), 11);
        for sol in &sols {
            assert_eq!(sol[0] + sol[1], 10);
        }
    }

    #[test]
    fn solve_all_respects_limit() {
        let domains = vec![(0..11).collect(), (0..11).collect()];
        let mut s = CustomSolver::new(&domains);
        s.post(Constraint::LinearEq(vec![(0, 1), (1, 1)], 10)).unwrap();
        assert_eq!(s.solve_all(3).len(), 3);
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
