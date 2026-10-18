# pyccp Examples Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `solve_all()` enumeration, eight tier-B propagators, and 16 runnable examples that assert their own results.

**Architecture:** Same hybrid as v1 — Rust owns the store, propagators, and search; Python owns the agent layer and public API. Examples are runnable scripts with assertions, executed by one parametrized pytest file. Enumeration is sequential-only (deterministic DFS order, no rayon) and does not notify the bridge.

**Tech Stack:** Rust 2021, pyo3 0.30 (extension-module), rayon 1; Python ≥ 3.11, pytest; maturin.

**Spec:** `docs/superpowers/specs/2026-10-18-pyccp-design.md`

## Global Constraints

- Python ≥ 3.11; Rust edition 2021.
- Rust deps: `pyo3 = "0.30"` (feature `extension-module`), `rayon = "1"`. No tokio.
- Python dev deps: `pytest` only. Build with `VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop`, test with `.venv/bin/pytest`.
- Integer (`i64`) domains only. No Python callables in the hot loop; constraints cross the FFI as data tuples.
- Propagators keep the v1 ceiling: prune only when all-but-one term is fixed, no bounds reasoning. Mark each with a `ponytail:` comment.
- `solve_all` is sequential-only and never notifies the bridge; agent-mode examples use `post` + `read`, enumeration examples use `solve_all`.
- Each example is a single file under 30 lines with a `run()` that asserts its own result and prints it.
- One failing check per unit; no fixtures, no frameworks beyond pytest and cargo test.
- Commit after every task.

## File Structure

```
rust/src/propagators.rs     # +8 propagator structs
rust/src/solver.rs          # +solve_all on the Solver trait
rust/src/bindings.rs        # +8 constraint kinds, +solve_all pymethod
pyccp/__init__.py           # +Store.solve_all
examples/
  __init__.py               # empty, makes examples importable
  simple.py  search.py  diophantine.py  modulo_filter.py  scaling.py
  element.py  cardinality.py  boolean_logic.py  cyclic.py
  nqueens.py  send_more_money.py  graph_coloring.py  magic_square.py
  agent_counter.py  agent_counter_random.py  producer_consumer.py
tests/test_examples.py
tests/test_solve_all.py
docs/EXAMPLES.md            # Status column added
```

---

### Task 1: `solve_all(limit)` enumeration

**Files:**
- Modify: `rust/src/solver.rs`
- Modify: `rust/src/bindings.rs`
- Modify: `pyccp/__init__.py`
- Create: `tests/test_solve_all.py`

**Interfaces:**
- Consumes: `Store`, `Registry`, `CustomSolver` from v1.
- Produces: `Solver::solve_all(&mut self, limit: usize) -> Vec<Vec<i64>>`; `pyccp.Store.solve_all(limit: int = 1000) -> list[list[int]]`.

- [ ] **Step 1: Write the failing Rust test in `rust/src/solver.rs`**

```rust
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
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cargo test solve_all` (from `rust/`)
Expected: compile error, `solve_all` not defined.

- [ ] **Step 3: Implement `solve_all` in `rust/src/solver.rs`**

Add to the `Solver` trait:

```rust
fn solve_all(&mut self, limit: usize) -> Vec<Vec<i64>>;
```

Implement on `CustomSolver` (sequential DFS, collect, stop at limit — no rayon, no bridge notification):

```rust
fn solve_all(&mut self, limit: usize) -> Vec<Vec<i64>> {
    let mut out = Vec::new();
    self.collect(&mut out, limit);
    out
}
```

and on the `impl CustomSolver` block:

```rust
// Returns true when the limit is reached and the search must unwind.
fn collect(&mut self, out: &mut Vec<Vec<i64>>, limit: usize) -> bool {
    if out.len() >= limit { return true; }
    let cell = match self.store.first_open() {
        Some(c) => c,
        None => { out.push(self.store.assignment()); return out.len() >= limit; }
    };
    let domain: Vec<i64> = self.store.domain(cell).to_vec();
    for &v in &domain {
        let m = self.store.mark();
        if self.store.commit(cell, v).is_ok()
            && self.registry.fixpoint(&mut self.store).is_ok()
            && self.collect(out, limit) {
            self.store.undo_to(m);
            return true;
        }
        self.store.undo_to(m);
    }
    false
}
```

- [ ] **Step 4: Add the pymethod in `rust/src/bindings.rs`**

```rust
fn solve_all(&mut self, limit: usize) -> Vec<Vec<i64>> {
    self.solver.solve_all(limit)
}
```

No `notify` call — enumeration does not cross the bridge.

- [ ] **Step 5: Write the failing Python test `tests/test_solve_all.py`**

```python
import pyccp

def test_solve_all_returns_all_pairs():
    s = pyccp.Store([list(range(11)), list(range(11))])
    s.post(("linear_eq", ([(0, 1), (1, 1)], 10)))
    sols = s.solve_all()
    assert len(sols) == 11
    assert all(a + b == 10 for a, b in sols)

def test_solve_all_limit():
    s = pyccp.Store([list(range(11)), list(range(11))])
    s.post(("linear_eq", ([(0, 1), (1, 1)], 10)))
    assert len(s.solve_all(limit=3)) == 3
```

- [ ] **Step 6: Implement `Store.solve_all` in `pyccp/__init__.py`**

```python
def solve_all(self, limit: int = 1000) -> list[list[int]]:
    """All solutions in DFS order. Does not notify the bridge: enumeration
    is branch-local, so mirrors cannot reflect it."""
    return self._core.solve_all(limit)
```

- [ ] **Step 7: Run both suites**

Run: `cargo test` (from `rust/`), then `VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop && .venv/bin/pytest tests/ -v`
Expected: cargo 15 + 2 = 17 passed; pytest 11 + 2 = 13 passed. Output pristine.

- [ ] **Step 8: Commit**

```bash
git add rust/ pyccp/ tests/
git commit -m "feat: solve_all enumeration (sequential, no bridge notification)"
```

---

### Task 2: Tier-B propagators (batched)

**Files:**
- Modify: `rust/src/propagators.rs`
- Modify: `rust/src/solver.rs` (`Constraint` enum + `to_propagator`)
- Modify: `rust/src/bindings.rs` (`parse_constraint`)
- Modify: `tests/test_examples.py` (created in Task 3 — this task adds only the propagator tests in `rust/src/propagators.rs`)

**Interfaces:**
- Consumes: `Store`, `Conflict`, `Propagator`, `Registry` from v1.
- Produces: structs `LessThan`, `Multiply`, `Element`, `Cardinality`, `Modulo`, `And`, `Or`, `Xor` implementing `Propagator`; `Constraint` enum gains `LessThan(usize, usize)`, `Multiply(usize, usize, i64)`, `Element(Vec<usize>, usize, usize)`, `Cardinality(Vec<usize>, i64, usize)`, `Modulo(usize, i64, i64)`, `And(usize, usize, usize)`, `Or(usize, usize, usize)`, `Xor(usize, usize, usize)`; Python kinds `"lt"`, `"mul"`, `"element"`, `"cardinality"`, `"modulo"`, `"and"`, `"or"`, `"xor"`.

- [ ] **Step 1: Write the failing tests in `rust/src/propagators.rs`**

```rust
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
    let mut s = Store::new(&[vec![0], vec![0, 1], vec![0]]);
    let mut r = Registry::new();
    r.add(Box::new(Or::new(0, 1, 2)));
    r.fixpoint(&mut s).unwrap();
    assert_eq!(s.domain(1), &[1]);
}

#[test]
fn xor_prunes() {
    let mut s = Store::new(&[vec![1], vec![0, 1], vec![1]]);
    let mut r = Registry::new();
    r.add(Box::new(Xor::new(0, 1, 2)));
    r.fixpoint(&mut s).unwrap();
    assert_eq!(s.domain(1), &[0]);
}
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cargo test` (from `rust/`)
Expected: compile error, propagator types not defined.

- [ ] **Step 3: Implement the eight propagators in `rust/src/propagators.rs`**

Each keeps the v1 ceiling — single-fixed-term pruning, marked `ponytail:`.

```rust
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
        let [x, y, z] = self.cells;
        let fixed = |c: usize| store.is_fixed(c);
        if fixed(x) && fixed(y) { store.commit(z, store.value(x) ^ store.value(y))?; }
        else if fixed(x) && fixed(z) { store.commit(y, store.value(x) ^ store.value(z))?; }
        else if fixed(y) && fixed(z) { store.commit(x, store.value(y) ^ store.value(z))?; }
        Ok(())
    }
}
```

- [ ] **Step 4: Extend `Constraint` and `to_propagator` in `rust/src/solver.rs`**

```rust
#[derive(Clone, Debug)]
pub enum Constraint {
    NeqLinear(Vec<(usize, i64)>, i64),
    AllDiff(Vec<usize>),
    LinearEq(Vec<(usize, i64)>, i64),
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
```

Add the eight match arms in `to_propagator`, e.g.:

```rust
Constraint::LessThan(i, j) => Box::new(LessThan::new(*i, *j)),
Constraint::Multiply(i, j, k) => Box::new(Multiply::new(*i, *j, *k)),
Constraint::Element(a, i, y) => Box::new(Element::new(a.clone(), *i, *y)),
Constraint::Cardinality(c, v, n) => Box::new(Cardinality::new(c.clone(), *v, *n)),
Constraint::Modulo(c, m, k) => Box::new(Modulo::new(*c, *m, *k)),
Constraint::And(x, y, z) => Box::new(And::new(*x, *y, *z)),
Constraint::Or(x, y, z) => Box::new(Or::new(*x, *y, *z)),
Constraint::Xor(x, y, z) => Box::new(Xor::new(*x, *y, *z)),
```

- [ ] **Step 5: Extend `parse_constraint` in `rust/src/bindings.rs`**

Python tuple shapes: `("lt", (i, j))`, `("mul", (i, j, k))`, `("element", ([array_cells], idx, y))`, `("cardinality", ([cells], value, count))`, `("modulo", (cell, m, k))`, `("and", (x, y, z))`, `("or", (x, y, z))`, `("xor", (x, y, z))`.

```rust
"lt" => Ok(Constraint::LessThan(args.get_item(0)?.extract()?, args.get_item(1)?.extract()?)),
"mul" => Ok(Constraint::Multiply(
    args.get_item(0)?.extract()?, args.get_item(1)?.extract()?, args.get_item(2)?.extract()?)),
"element" => Ok(Constraint::Element(
    args.get_item(0)?.extract()?, args.get_item(1)?.extract()?, args.get_item(2)?.extract()?)),
"cardinality" => Ok(Constraint::Cardinality(
    args.get_item(0)?.extract()?, args.get_item(1)?.extract()?, args.get_item(2)?.extract()?)),
"modulo" => {
    let m: i64 = args.get_item(1)?.extract()?;
    if m == 0 { return Err(PyValueError::new_err("modulo divisor must be non-zero")); }
    Ok(Constraint::Modulo(
        args.get_item(0)?.extract()?, m, args.get_item(2)?.extract()?))
}
"and" | "or" | "xor" => {
    let kind = kind.clone();
    Ok(match kind.as_str() {
        "and" => Constraint::And(args.get_item(0)?.extract()?, args.get_item(1)?.extract()?, args.get_item(2)?.extract()?),
        "or"  => Constraint::Or(args.get_item(0)?.extract()?, args.get_item(1)?.extract()?, args.get_item(2)?.extract()?),
        _     => Constraint::Xor(args.get_item(0)?.extract()?, args.get_item(1)?.extract()?, args.get_item(2)?.extract()?),
    })
}
```

Keep the existing bounds check (`check` closure) applied to every cell index in the new kinds.

- [ ] **Step 6: Run tests, verify they pass**

Run: `cargo test` (from `rust/`)
Expected: 17 + 14 = 31 passed, no warnings.

- [ ] **Step 7: Commit**

```bash
git add rust/
git commit -m "feat: tier-B propagators (lt, mul, element, cardinality, modulo, and/or/xor)"
```

---

### Task 3: Tier-A examples

**Files:**
- Create: `examples/__init__.py` (empty)
- Create: `examples/nqueens.py`, `examples/send_more_money.py`, `examples/graph_coloring.py`, `examples/magic_square.py`, `examples/diophantine.py`, `examples/modulo_filter.py`
- Create: `tests/test_examples.py`

**Interfaces:**
- Consumes: `pyccp.Store`, `Store.solve`, `Store.solve_all`, constraint tuples.
- Produces: `run()` per example returning its solution(s); `tests/test_examples.py` parametrized over example names.

- [ ] **Step 1: Write the failing test `tests/test_examples.py`**

```python
import importlib
import pytest

EXAMPLES = [
    "nqueens", "send_more_money", "graph_coloring", "magic_square",
    "diophantine", "modulo_filter",
]

@pytest.mark.parametrize("name", EXAMPLES)
def test_example(name):
    mod = importlib.import_module(f"examples.{name}")
    mod.run()
```

- [ ] **Step 2: Run test, verify it fails**

Run: `.venv/bin/pytest tests/test_examples.py -v`
Expected: ModuleNotFoundError for `examples.nqueens`.

- [ ] **Step 3: Implement the six examples**

`examples/nqueens.py` — diagonals use `NeqLinear` with `k = i - j` and `k = j - i`, not plain `neq`:

```python
from pyccp import Store

def run():
    n = 8
    s = Store([list(range(n)) for _ in range(n)])
    for i in range(n):
        for j in range(i + 1, n):
            s.post(("neq", (i, j)))
            s.post(("neq_linear", ([(i, 1), (j, -1)], i - j)))
            s.post(("neq_linear", ([(i, 1), (j, -1)], j - i)))
    sol = s.solve()
    assert sol is not None
    for i in range(n):
        for j in range(i + 1, n):
            assert sol[i] != sol[j]
            assert sol[i] - i != sol[j] - j
            assert sol[i] + i != sol[j] + j
    print("nqueens:", sol)
    return sol
```

`parse_constraint` must also accept `"neq_linear"` as `("neq_linear", ([(idx, coef)], k))` — add this kind in this task (one match arm, same shape as `linear_eq`).

`examples/send_more_money.py`:

```python
from pyccp import Store

def run():
    s = Store([list(range(10))] * 8)          # S E N D M O R Y
    s.post(("all_diff", (list(range(8)),)))
    s.post(("nonzero", (0,)))
    s.post(("nonzero", (4,)))
    s.post(("linear_eq", ([(0, 1000), (1, 100), (2, 10), (3, 1),
                            (4, 1000), (5, 100), (6, 10), (1, 1),
                            (4, -10000), (5, -1000), (2, -100), (1, -10), (7, -1)], 0)))
    sol = s.solve()
    assert sol == [9, 5, 6, 7, 1, 0, 8, 2]
    print("send_more_money:", sol)
    return sol
```

`examples/graph_coloring.py`:

```python
from pyccp import Store

def run():
    # K4: every pair of 4 nodes differs, 3 colours
    s = Store([list(range(3))] * 4)
    for i in range(4):
        for j in range(i + 1, 4):
            s.post(("neq", (i, j)))
    sol = s.solve()
    assert sol is not None and len(set(sol)) == 4
    print("graph_coloring:", sol)
    return sol
```

`examples/magic_square.py`:

```python
from pyccp import Store

def run():
    # 3x3 magic square, cells 0..8, every line sums to 15
    lines = [(0,1,2), (3,4,5), (6,7,8), (0,3,6), (1,4,7), (2,5,8), (0,4,8), (2,4,6)]
    s = Store([list(range(1, 10))] * 9)
    s.post(("all_diff", (list(range(9)),)))
    for line in lines:
        s.post(("linear_eq", ([(c, 1) for c in line], 15)))
    sol = s.solve()
    assert sol is not None
    for line in lines:
        assert sum(sol[c] for c in line) == 15
    print("magic_square:", sol)
    return sol
```

`examples/diophantine.py`:

```python
from pyccp import Store

def run():
    s = Store([list(range(11)), list(range(11))])
    s.post(("linear_eq", ([(0, 1), (1, 1)], 10)))
    sols = s.solve_all()
    assert len(sols) == 11
    assert all(a + b == 10 for a, b in sols)
    print("diophantine:", sols)
    return sols
```

`examples/modulo_filter.py` — "not divisible" is domain filtering in Python; `Modulo` handles the divisible side:

```python
from pyccp import Store

def run():
    domain = [v for v in range(1, 101) if v % 4 != 0 and v % 7 != 0]
    s = Store([domain])
    s.post(("modulo", (0, 3, 0)))
    sols = s.solve_all()
    assert len(sols) == 22
    assert all(v[0] % 3 == 0 for v in sols)
    print("modulo_filter:", len(sols), "solutions")
    return sols
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop && .venv/bin/pytest tests/ -v`
Expected: 6 example tests pass; existing 13 still pass.

- [ ] **Step 5: Commit**

```bash
git add examples/ tests/
git commit -m "feat: tier-A examples (nqueens, sendmore, coloring, magic square, diophantine, modulo)"
```

---

### Task 4: Tier-B examples

**Files:**
- Create: `examples/simple.py`, `examples/search.py`, `examples/scaling.py`, `examples/element.py`, `examples/cardinality.py`, `examples/boolean_logic.py`, `examples/cyclic.py`
- Modify: `tests/test_examples.py` (extend `EXAMPLES`)

**Interfaces:**
- Consumes: the eight propagators from Task 2, `solve_all` from Task 1.

- [ ] **Step 1: Extend `tests/test_examples.py`**

```python
EXAMPLES = [
    "nqueens", "send_more_money", "graph_coloring", "magic_square",
    "diophantine", "modulo_filter",
    "simple", "search", "scaling", "element", "cardinality",
    "boolean_logic", "cyclic",
]
```

- [ ] **Step 2: Run tests, verify the seven new ones fail**

Run: `.venv/bin/pytest tests/test_examples.py -v`
Expected: ModuleNotFoundError for the seven new names.

- [ ] **Step 3: Implement the seven examples**

`examples/simple.py`:

```python
from pyccp import Store

def run():
    s = Store([list(range(1, 4)), list(range(1, 4))])
    s.post(("lt", (0, 1)))
    sol = s.solve()
    assert sol is not None and sol[0] < sol[1]
    print("simple:", sol)
    return sol
```

`examples/search.py`:

```python
from pyccp import Store

def run():
    # asymmetric domains, x < y, enumerate
    s = Store([list(range(1, 4)), list(range(3, 6))])
    s.post(("lt", (0, 1)))
    sols = s.solve_all()
    assert len(sols) == 8            # (1,3)..(1,5), (2,3)..(2,5), (3,4), (3,5)
    assert all(a < b for a, b in sols)
    print("search:", sols)
    return sols
```

`examples/scaling.py`:

```python
from pyccp import Store

def run():
    s = Store([list(range(1, 13)), list(range(1, 13))])
    s.post(("mul", (0, 1, 12)))
    sol = s.solve()
    assert sol is not None and sol[0] * sol[1] == 12
    print("scaling:", sol)
    return sol
```

`examples/element.py`:

```python
from pyccp import Store

def run():
    # y == array[idx], array = cells 0..2, idx = 3, y = 4
    s = Store([list(range(1, 4))] * 3 + [[0, 1, 2], list(range(1, 4))])
    s.post(("element", ([0, 1, 2], 3, 4)))
    s.post(("lt", (3, 4)))            # idx < y, so the answer is not trivial
    sol = s.solve()
    assert sol is not None and sol[4] == sol[sol[3]]
    print("element:", sol)
    return sol
```

`examples/cardinality.py`:

```python
from pyccp import Store

def run():
    s = Store([list(range(1, 5))] * 5)
    s.post(("cardinality", (list(range(5)), 3, 2)))
    sol = s.solve()
    assert sol is not None and sol.count(3) == 2
    print("cardinality:", sol)
    return sol
```

`examples/boolean_logic.py`:

```python
from pyccp import Store

def run():
    # x ∧ y = z, x ∨ y = w, x ⊕ y = v over 0/1
    s = Store([[0, 1]] * 5)           # x y z w v
    s.post(("and", (0, 1, 2)))
    s.post(("or", (0, 1, 3)))
    s.post(("xor", (0, 1, 4)))
    s.post(("neq", (0, 1)))           # x != y -> xor is 1
    sol = s.solve()
    assert sol is not None
    assert sol[2] == sol[0] * sol[1]
    assert sol[3] == min(sol[0] + sol[1], 1)
    assert sol[4] == sol[0] ^ sol[1]
    print("boolean_logic:", sol)
    return sol
```

`examples/cyclic.py`:

```python
from pyccp import Store

def run():
    # shift type = day mod 3 for days 1..7
    s = Store([list(range(1, 8))])
    s.post(("modulo", (0, 3, 1)))
    sol = s.solve()
    assert sol is not None and sol[0] % 3 == 1
    print("cyclic:", sol)
    return sol
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `.venv/bin/pytest tests/ -v`
Expected: 13 example tests pass, existing tests still pass.

- [ ] **Step 5: Commit**

```bash
git add examples/ tests/
git commit -m "feat: tier-B examples (simple, search, scaling, element, cardinality, boolean, cyclic)"
```

---

### Task 5: Agent examples

**Files:**
- Create: `examples/producer_consumer.py`, `examples/agent_counter.py`, `examples/agent_counter_random.py`
- Modify: `tests/test_examples.py` (extend `EXAMPLES`)

**Interfaces:**
- Consumes: `pyccp.Store`, `Var`, `join`, `choice`, `dispatcher` from v1. No propagators beyond `NeqLinear`/`LinearEq`.

- [ ] **Step 1: Extend `tests/test_examples.py`**

```python
EXAMPLES = [
    "nqueens", "send_more_money", "graph_coloring", "magic_square",
    "diophantine", "modulo_filter",
    "simple", "search", "scaling", "element", "cardinality",
    "boolean_logic", "cyclic",
    "producer_consumer", "agent_counter", "agent_counter_random",
]
```

- [ ] **Step 2: Run tests, verify the three new ones fail**

Run: `.venv/bin/pytest tests/test_examples.py -v`
Expected: ModuleNotFoundError for the three new names.

- [ ] **Step 3: Implement the three examples**

`examples/producer_consumer.py`:

```python
import asyncio
import pyccp

def run():
    async def main():
        s = pyccp.Store([[1, 2], [2, 3]])
        x, y = s.var("x", 0), s.var("y", 1)
        s.post(("linear_eq", ([(1, 1), (0, -1)], 1)))   # y = x + 1
        seen = []

        async def producer():
            await x.commit(1)

        async def consumer():
            seen.append(await y.read())

        d = asyncio.create_task(pyccp.dispatcher(s))
        await pyccp.join(producer, consumer)
        d.cancel()
        assert seen == [2]
        print("producer_consumer:", seen)
        return seen

    return asyncio.run(main())
```

`examples/agent_counter.py`:

```python
import asyncio
import pyccp

def run():
    async def main():
        n = 4
        s = pyccp.Store([list(range(n + 1))] * n)
        ports = [s.var(f"p{i}", i) for i in range(n)]
        seen = []

        async def agent(i):
            v = await ports[i].read()
            seen.append(i)
            await ports[(i + 1) % n].commit(v + 1)

        d = asyncio.create_task(pyccp.dispatcher(s))
        await ports[0].commit(0)
        await pyccp.join(*[lambda i=i: agent(i) for i in range(n)])
        d.cancel()
        assert seen == [0, 1, 2, 3]
        print("agent_counter:", seen)
        return seen

    return asyncio.run(main())
```

`join` takes agent functions of zero args, so the lambda-per-index form is required; keep it as written.

`examples/agent_counter_random.py`:

```python
import asyncio
import random
import pyccp

def run():
    async def main():
        random.seed(7)
        n = 4
        s = pyccp.Store([list(range(n + 1))] * n)
        ports = [s.var(f"p{i}", i) for i in range(n)]
        seen = []

        async def agent(i):
            v = await ports[i].read()
            seen.append(i)
            await ports[random.randrange(n)].commit(v + 1)

        d = asyncio.create_task(pyccp.dispatcher(s))
        await ports[0].commit(0)
        await pyccp.join(*[lambda i=i: agent(i) for i in range(n)])
        d.cancel()
        assert seen == [0, 2, 1, 3]
        print("agent_counter_random:", seen)
        return seen

    return asyncio.run(main())
```

A random ring can deadlock (an agent forwards to a port whose agent already ran) or conflict (re-committing a fixed port). Seed with `random.seed(7)`, verified to produce the full walk `0 → 2 → 1 → 3`; assert that exact walk. If the seed is changed, re-verify the walk covers all four agents.

- [ ] **Step 4: Run tests, verify they pass**

Run: `.venv/bin/pytest tests/ -v`
Expected: 16 example tests pass, existing tests still pass.

- [ ] **Step 5: Commit**

```bash
git add examples/ tests/
git commit -m "feat: agent examples (producer_consumer, agent_counter, agent_counter_random)"
```

---

### Task 6: Update `docs/EXAMPLES.md`

**Files:**
- Modify: `docs/EXAMPLES.md`

- [ ] **Step 1: Add a Status column**

Rewrite the table with `Status` values `done` for the 16 implemented examples and `v2` for the rest, each with a one-line reason:

- `agent_inbox_monitoring` — v2: no inbox introspection in v1
- `names_demo` — v2: no names API in v1
- `knapsack`, `scheduling`, `tsp`, `linear` — v2: optimization / LP backend
- `circuit`, `parallel_sudoku` — v2: specialized propagators (Hamiltonian cycle, AC-4)

Add the three new examples that were not in the original list (`graph_coloring`, `magic_square`, `producer_consumer`) as `done`.

- [ ] **Step 2: Commit**

```bash
git add docs/EXAMPLES.md
git commit -m "docs: EXAMPLES.md status column"
```

---

## Self-Review

**Spec coverage:** `solve_all` (T1), eight tier-B propagators (T2), tier-A examples (T3), tier-B examples (T4), agent examples (T5), status doc (T6). Deferred items (optimization, AC-4, GLOP, names API, inbox introspection) have no tasks by design.

**Placeholder scan:** none — every step carries its code.

**Type consistency:** `Constraint` enum variants match `to_propagator` and `parse_constraint`; Python tuple shapes match the examples; `run()` returns solutions and asserts internally; `tests/test_examples.py` imports `examples.<name>` and calls `run()`.

**Known ceilings:** all new propagators keep the single-fixed-term pruning ceiling, marked `ponytail:`; `solve_all` is sequential-only and does not notify the bridge.

**Two counts to verify at execution time, not assumed:** `search` yields 8 pairs (not 3), `modulo_filter` yields 22 values — both asserted in the example code; adjust the assertion if the run disagrees.
