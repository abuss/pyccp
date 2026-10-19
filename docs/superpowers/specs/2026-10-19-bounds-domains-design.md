# Design Document: Phase B (v2.5.1) — Bounds-Based Domains & MKP Scaling

**Date:** 2026-10-19  
**Phase:** v2.5.1 (after v2.5.0 Element2D)  
**Scope:** Core Store redesign + propagator updates  
**Estimated Effort:** 10–12 hours  
**Risk Level:** Medium (core changes, but incremental approach mitigates)

---

## Executive Summary

Phase B implements a hybrid domain representation (Vec + Bounds) to enable solving large combinatorial problems like MKP with m=30 items. The Bounds variant compactly represents domains as (min, max) pairs, avoiding memory exhaustion and enabling efficient propagation via interval arithmetic.

**Success Criterion:** pb4.dat (m=29, capacity ~50,000) solves in < 30 seconds.

**Approach:**
1. Add Domain::Bounds variant to store.rs (unified API for both variants)
2. Update LinearEq, LeLinear, AllDiff propagators for bounds reasoning
3. Implement advanced AllDiff logic (disjoint intervals, pigeonhole principle)
4. Auto-convert large domains to Bounds at Store creation (threshold-based)
5. Integrate into MKP example; benchmark against v2.4

---

## 1. Goals & Constraints

### Goals
- ✅ Enable MKP for m=30 items (vs. m~20 in v2.4)
- ✅ Solve pb4.dat in < 30 seconds
- ✅ Maintain backward compatibility (all 42 baseline tests pass)
- ✅ Support hybrid Vec/Bounds domains in same solver
- ✅ Provide bounds-aware propagators for LinearEq, LeLinear, AllDiff

### Constraints
- All existing Vec domain behavior unchanged
- Trail system extended but remains O(1) per undo
- Propagators must handle both domain types gracefully
- No new external dependencies
- Performance: no regression on Vec domains

---

## 2. Architecture

### 2.1 Domain Representation

**Current (v2.4):**
```rust
pub enum Domain {
    Vec(Vec<i64>),  // only option
}
```

**New (v2.5.1):**
```rust
pub enum Domain {
    Vec(Vec<i64>),      // Enumerated domain (sorted, dedup'd)
    Bounds(i64, i64),   // Compact (min, max) pair
}
```

**When to use each:**
- **Vec:** Small domains (≤ 1000 values), allows precise enumeration
- **Bounds:** Large domains (> 1000 values), memory-efficient, enables bound reasoning

**Threshold:** Configurable (default: 1000 values). Store::new() auto-converts domains exceeding threshold to Bounds.

### 2.2 Unified API

All Domain methods work with both variants:

| Method | Vec | Bounds | Complexity |
|--------|-----|--------|-----------|
| `is_fixed()` | len() == 1 | min == max | O(1) |
| `value()` | vec[0] | min (assert min==max) | O(1) |
| `contains(v)` | binary_search | min ≤ v ≤ max | O(log n) / O(1) |
| `domain()` | return Vec | return None/error | Varies |
| `bounds()` | compute min/max | return stored (min, max) | O(n) / O(1) |
| `remove(cell, v)` | Remove from vec | Tighten if at boundary | See below |
| `tighten_bounds(cell, min, max)` | Prune vec values | Update bounds | O(n) / O(1) |

### 2.3 remove() on Bounds Domains

**Problem:** Removing arbitrary interior values from Bounds cannot be represented.

**Solution:**
- `remove(v)` on Bounds returns false if v is interior; only succeeds at boundaries
- Boundary removal tightens bounds:
  - `remove(1)` on Bounds(1,10) → tighten to Bounds(2,10)
  - `remove(10)` on Bounds(1,10) → tighten to Bounds(1,9)
  - `remove(5)` on Bounds(1,10) → returns false (cannot represent)

**Propagator consequence:** When propagators need to prune interior values:
- Use `tighten_bounds()` if you know the new valid range
- Use domain intersection for complex pruning (rare case; mostly for bounds anyway)

### 2.4 Trail Extension

**Current:** Records (cell, old_vec) for undo.

**New:** Records (cell, old_domain) where old_domain is full enum.

**Behavior:**
- When undoing, restore entire Domain variant
- Undo works correctly whether cell was Vec or Bounds at checkpoint

---

## 3. Store Implementation (store.rs)

### 3.1 Changes to store.rs

**File:** `rust/src/store.rs`  
**Sections:**
- Lines 31–45: Domain enum definition
- Lines 46–150: Domain impl methods (is_fixed, value, contains, bounds, remove, tighten_bounds)
- Lines 200–250: Store::new() threshold logic
- Lines 280–320: Trail extension for Bounds

### 3.2 API Additions for Propagators

```rust
impl Store {
    // New query methods
    pub fn is_bounds_domain(&self, cell: usize) -> bool
    pub fn is_vec_domain(&self, cell: usize) -> bool
    pub fn domain_or_none(&self, cell: usize) -> Option<Vec<i64>>
    
    // Extend existing methods
    pub fn tighten_bounds(&mut self, cell: usize, new_min: i64, new_max: i64) -> Result<(), Conflict>
    pub fn bounds(&self, cell: usize) -> Bounds  // Works for both Vec and Bounds
}
```

### 3.3 Auto-Conversion Heuristic

```rust
pub fn new(domains: Vec<Vec<i64>>) -> Self {
    const THRESHOLD: usize = 1000;  // tunable
    
    let domains: Vec<Domain> = domains.into_iter().map(|vec| {
        if vec.is_empty() {
            return Err(Conflict);
        }
        if vec.len() > THRESHOLD {
            let min = *vec.iter().min().unwrap();
            let max = *vec.iter().max().unwrap();
            Domain::Bounds(min, max)
        } else {
            Domain::Vec(vec)
        }
    }).collect()?;
    
    Store { cells: domains, trail: vec![], ... }
}
```

---

## 4. Propagator Updates

### 4.1 LinearEq (sum(coeff[i] * var[i]) == rhs)

**Current flow:** Bounds check → all-but-one logic.

**New flow:**
1. **Bounds check** (unchanged): min_sum ≤ rhs ≤ max_sum
2. **All-but-one fixed** (enhanced):
   - If open term is Vec: enumerate and prune (existing behavior)
   - If open term is Bounds: solve and tighten
   - If fixed: `v = (rhs - sum_fixed) / coeff`, call `tighten_bounds(v, v, v)`

**Pseudocode:**
```rust
if open.len() == 1 {
    let (c, k) = open[0];
    let sum = self.rhs - sum_fixed;
    let v = sum / k;
    
    if store.is_bounds_domain(c) {
        // Tighten bounds
        store.tighten_bounds(c, v, v)?;
    } else {
        // Existing Vec logic: enumerate and remove non-matching values
        for val in store.domain(c).to_vec() {
            if val != v {
                store.remove(c, val)?;
            }
        }
    }
}
```

### 4.2 LeLinear (sum(coeff[i] * var[i]) ≤ rhs)

**Similar to LinearEq but one-sided:**
- If max_sum ≤ rhs: constraint satisfied
- If all-but-one fixed: solve and tighten upper bound only
- If multiple open and max_sum > rhs: tighten all vars' upper bounds

**Pseudocode:**
```rust
let max_sum = self.compute_max_sum(store);
if max_sum > self.rhs {
    // Constraint violated; tighten upper bounds
    for (c, k) in &self.terms {
        if !store.is_fixed(*c) {
            let available = self.rhs - (max_sum - bounds(c).max * k);
            let upper = available / k;
            store.tighten_bounds(*c, bounds(c).min, upper)?;
        }
    }
}
```

### 4.3 AllDiff (all vars distinct)

**Current flow:** Enumerate each domain, mark used values, prune.

**New flow with Bounds reasoning (disjoint intervals):**

1. **Collect intervals:**
   - For each var, get bounds (from Vec min/max or Bounds)
   - Sort by (min, max)

2. **Detect disjoint intervals:**
   - If max[i] < min[i+1] → intervals disjoint, no overlap
   - If intervals overlap → potential conflict

3. **Pigeonhole check:**
   - For overlapping region [a, b]: count vars k with bounds intersecting [a, b]
   - If k > (b - a + 1) → **Conflict** (cannot assign k distinct values to b-a+1 slots)
   - If k == (b - a + 1) → **Tighten:** all k vars must use exactly [a, b]

4. **Propagate (only to Vec domains):**
   - If a Bounds domain was tightened to [a, b], update it
   - If a Vec domain overlaps with tightened interval, prune values outside interval

**Pseudocode:**
```rust
fn propagate_bounds_reasoning(&mut self, store: &mut Store) -> Result<(), Conflict> {
    let mut intervals: Vec<(i64, i64, usize)> = self.cells.iter()
        .map(|c| {
            let b = store.bounds(*c);
            (b.min, b.max, *c)
        })
        .collect();
    
    intervals.sort_by_key(|x| (x.0, x.1));
    
    // Check for overlaps and pigeonhole principle
    let mut i = 0;
    while i < intervals.len() {
        let (min_a, max_a, _) = intervals[i];
        let mut count = 1;
        let mut j = i + 1;
        
        while j < intervals.len() {
            let (min_b, max_b, _) = intervals[j];
            if min_b <= max_a {
                // Overlapping
                count += 1;
                j += 1;
            } else {
                break;
            }
        }
        
        // Check pigeonhole for region [min_a, max_a]
        let available = max_a - min_a + 1;
        if count > available {
            return Err(Conflict);  // Too many vars in too small a region
        }
        
        if count == available {
            // Tighten all overlapping vars to [min_a, max_a]
            for k in i..j {
                store.tighten_bounds(intervals[k].2, min_a, max_a)?;
            }
        }
        
        i = j;
    }
    
    Ok(())
}
```

---

## 5. Store Integration

### 5.1 MKP Example (examples/mknapsack.py)

**No API changes required.** Store::new() auto-converts large domains.

**Example:**
```python
# pb4.dat: 29 items, capacity ~50,000
# Old: Creates Vec(0..50000) = 50001 values → memory exhaustion
# New: Auto-converts to Bounds(0, 50000) = O(1) space

domains = [
    [0, 1] for _ in range(29)           # item selections (small, stays Vec)
    + [[0, 50000] for _ in range(6)]    # weight sums (large, converts to Bounds)
    + [[0, 10000000]]                    # value sum (large, converts to Bounds)
]

store = Store(domains)  # Threshold auto-conversion happens here
```

### 5.2 Testing

**Unit tests (rust/tests/bounds_domains.rs):**
- Domain API: is_fixed, value, contains, bounds, remove, tighten_bounds (3 tests)
- Store hybrid: mixed Vec/Bounds, trail undo, threshold conversion (3 tests)
- LinearEq bounds: conflict, all-but-one, tighten (4 tests)
- LeLinear bounds: conflict, tighten (2 tests)
- AllDiff bounds: disjoint intervals, pigeonhole, exact fit (3 tests)

**Integration tests:**
- `test_mknapsack_pb4_under_30s()`: Performance benchmark
- `test_mknapsack_bounds_vs_vec()`: Correctness (same optimal value)

**Regression tests:**
- All 42 baseline tests must pass

---

## 6. Implementation Plan

**Phases (incremental, with checkpoints):**

| Phase | Task | Hours | Checkpoint |
|-------|------|-------|-----------|
| B1 | Domain::Bounds, Store API, threshold | 2-3 | `refactor: Domain::Bounds variant` |
| B2 | LinearEq bounds propagation | 2 | `feat: LinearEq bounds-only propagation` |
| B3 | LeLinear bounds propagation | 1.5 | `feat: LeLinear bounds-only propagation` |
| B4 | AllDiff bounds reasoning (intervals) | 3 | `feat: AllDiff advanced bounds reasoning` |
| B5 | MKP integration & tuning | 1.5 | `feat: MKP optimization with bounds domains` |
| B6 | Docs & release notes | 1 | `docs: v2.5.1 release notes` |

**Total:** 11–12 hours

---

## 7. Risk & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Store changes break propagators | Medium | Critical | Test B1 thoroughly; checkpoint before B2 |
| AllDiff interval logic bugs | Medium | High | Unit test interval helpers separately; extra review |
| Bounds threshold suboptimal | Low | Medium | Make threshold configurable; benchmark and tune |
| pb4.dat still slow | Low | High | Profile; add dynamic pruning if needed |
| Regressions in baseline tests | Low | High | Run full suite after each phase |

**Mitigation strategy:** Incremental phases with checkpoints; regression tests after each phase.

---

## 8. Success Criteria

- ✅ pb4.dat solves in < 30 seconds (performance)
- ✅ All 42 baseline tests pass (backward compatibility)
- ✅ 15+ unit tests for bounds reasoning (correctness)
- ✅ Bounds domains work alongside Vec domains (hybrid support)
- ✅ Documentation updated with v2.5.1 release notes

---

## 9. Future Work (v3+)

- **Interval arithmetic:** Further optimize bounds propagation
- **Adaptive threshold:** Dynamic threshold based on problem size
- **Parallel B&B:** Branch-and-bound with bounds domains
- **3D+ Element:** Extend Element2D bounds reasoning to higher dimensions

---

**Design approval required before proceeding to implementation.**
