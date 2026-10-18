# pyccp v1 Design — CCP Runtime with Pluggable Solver

**Date:** 2026-10-18
**Status:** approved (design agreed in brainstorming; this is the written spec)

## 1. Scope

pyccp v1 ships two production-grade halves, equally:

1. **Concurrent agents** — asyncio-based CCP runtime: dataflow variables, suspension until
   entailment, join (`|`), choice, deterministic monotone store.
2. **Synchronous search** — a constraint solver over integer domains: DFS with propagator
   fixpoint at each node, parallelized at split points.

The solver is pluggable through a Rust trait. v1 implements **only the custom solver**;
Z3, OR-Tools, and Gecode are deferred to v2 as implementations of the same trait.

**Non-goals for v1** (explicitly deferred, not stubbed):
- External solver backends (Z3/OR-Tools/Gecode) — trait only, no implementations.
- Python-callable propagators (slow path) — add when a real global constraint needs it.
- Global entailment across search branches.
- Cross-branch observation by agents.
- Enumeration of all solutions (`solve()` returns the first found).
- Float/string domains — integer (`i64`) domains only.

## 2. Architecture

Hybrid, PyO3 bridge:

```
Python layer (agents, API)              Rust core (hot loop)
──────────────────────────              ──────────────────────────
asyncio agents, Var handles             Store (domain cells, trail)
join / choice / at combinators          Propagator registry + fixpoint
dispatcher task (bridge)                Solver trait + CustomSolver (DFS, rayon)
```

- **Python owns** agent orchestration and the public API. Iteration on agent code needs no
  recompilation.
- **Rust owns** the store, propagators, and search. Propagators execute in Rust: they fire
  per domain change (millions of times), so no Python callable crosses the boundary in the
  hot loop. Constraints cross as **data**, not closures.
- **The Rust store is the single source of truth.** Python `Var` is a thin handle to a store
  cell, not a parallel store.

## 3. Components

### 3.1 Layout

```
pyccp/
├── rust/                    # single crate `pyccp-core`, cdylib via PyO3
│   ├── store.rs             # domain cells, trail (undo log), per-branch state
│   ├── propagators.rs       # Propagator trait + built-ins
│   ├── solver.rs            # Solver trait + CustomSolver (DFS + rayon)
│   └── bindings.rs          # PyStore, PyVar, notification path
└── pyccp/                   # pure Python package
    ├── __init__.py          # Store, Var public API, Conflict exception
    ├── dispatcher.py        # bridge drain loop
    └── agents.py            # join / choice / at combinators
```

No cargo workspace, no extra crates.

### 3.2 Store (Rust)

- `Store` holds `Vec<Cell>`; a `Cell` is an integer domain (sorted `Vec<i64>`) plus a fixed
  flag. An `Assignment` is `Vec<i64>` indexed by cell id.
- Backtracking uses a **trail**: every domain change is logged; undo pops the trail. No
  domain cloning.
- Per-branch state: each search branch owns its view of the cells through the trail.

### 3.3 Propagators (Rust)

```rust
pub trait Propagator: Send + Sync {
    fn cells(&self) -> &[usize];
    fn propagate(&mut self, store: &mut Store) -> Result<(), Conflict>;
}
```

- The store keeps a registry mapping cell → propagators. When a cell changes, its
  propagators run until fixpoint.
- Built-in propagators for v1: `NeqLinear(terms, k)` (sum(coef·v) != k), `AllDiff(indices)`,
  `LinearEq(terms, rhs)` (sum(coef·v) == rhs), `NonZero(i)`. N-Queens columns are
  `NeqLinear` with `k = 0`; diagonals are `NeqLinear` with `k = i - j` and `k = j - i` —
  no dedicated diagonal kind.
- A propagator prunes domains; pruning to the empty set is a `Conflict`.

### 3.4 Solver trait (pluggability boundary)

```rust
pub trait Solver {
    fn post(&mut self, c: Constraint) -> Result<(), Conflict>;
    fn entails(&self, c: &Constraint) -> bool;
    fn solve(&mut self) -> Option<Assignment>;
}
```

- `Constraint` is a data enum (the four built-in kinds above). This is the only interface a
  backend must implement; Z3/OR-Tools/Gecode implement it in v2 without touching anything
  else.
- `CustomSolver` is the only v1 implementation.

### 3.5 Search

- DFS; at each node, run propagators to fixpoint, then choose a non-fixed cell and split.
- Parallelism via **rayon at split points only** (top-level splits), each worker owning its
  own branch/trail; first solution wins.
- `ponytail:` no per-propagator parallelism — split-level parallelism is enough until
  profiling says otherwise.

### 3.6 Bridge (Rust → Python notification)

- When a cell becomes fixed (domain singleton), Rust pushes `(cell_id, value)` to a
  `queue.SimpleQueue` (thread-safe) and wakes the dispatcher via
  `loop.call_soon_threadsafe`.
- The dispatcher task drains the queue and sets the mirror `asyncio.Event` for the handle.
- Crossing happens **once per entailed fact**, never per propagation step.

### 3.7 Branch semantics

- Within a branch, the store is monotone: facts accumulate, results are
  scheduling-independent (CCP determinism).
- In v1, agents observe **only the main branch**. An agent that reads a solver variable
  during `solve()` is bound to the main branch; if the main branch backtracks past a fact
  the agent observed, the agent task is cancelled and its side effects are lost
  (Oz-faithful semantics).
- Parallel workers never notify: their branches are solver-private.
- `ponytail:` single-branch observation, no cross-branch entailment; add multi-branch
  observation when actually needed.

## 4. Data flow

```
agent: var.commit(42)
  → PyO3 → store cell narrowed → propagators fire (Rust) → fixpoint
  → fact entailed? → SimpleQueue + call_soon_threadsafe
  → dispatcher sets asyncio.Event
  → agent's `await var.read()` wakes
```

`solve()` is called once per problem from Python; the whole search stays in Rust.

## 5. Error handling

- Empty domain after propagation → `Conflict` → backtrack (search) or raise (agent mode).
- Double-commit with a different value → `Conflict`.
- Python sees `pyccp.Conflict` exceptions; no panics cross the FFI boundary.

## 6. Testing

- **Rust (`cargo test`):** propagator unit tests (domain pruning, conflict detection);
  golden problems — 8-Queens and SEND+MORE=MONEY — must produce the known solutions.
- **Python (`pytest`):** suspension/wake semantics, conflict detection, dispatcher bridge.
- **One FFI integration test:** producer agent commits, propagator prunes in Rust, consumer
  agent wakes through the bridge.
- **Parallel check:** 12-Queens solved by the rayon path must match the sequential solution
  set for the first solution.

One failing check per unit; no fixtures, no frameworks beyond pytest and cargo test.

## 7. Success criteria

1. Golden problems solve correctly through the Rust core.
2. Producer/consumer bridge test passes (incremental propagation visible to agents).
3. Conflicts raise `pyccp.Conflict`, not panics.
4. Parallel search measurably beats sequential DFS on 12-Queens.

## 8. Deferred (v2)

Z3, OR-Tools, Gecode as `Solver` implementations; Python-callable propagators; global
entailment; all-solutions enumeration; non-integer domains.
