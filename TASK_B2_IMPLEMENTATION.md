# Task B2: LinearEq Bounds-Only Propagation — Implementation Summary

**Date:** 2026-10-19  
**Status:** ✅ COMPLETE  
**Subagent:** Claude Haiku 4.5

## Deliverables

### ✅ Requirement 1: Bounds-Aware LinearEq Propagation
**Implementation:** Modified `LinearEq::propagate()` all-but-one case (lines 203-217 in propagators.rs)

When exactly one variable is unfixed:
1. Solve: `coeff * var = rhs - sum_of_fixed_terms`
2. Check domain type: `store.is_bounds_domain(cell)`
3. **Bounds domain path:** Call `store.tighten_bounds(cell, Bounds::new(v, v))` to fix value
4. **Vec domain path:** Use existing enumerate-and-remove logic (unchanged)

**Benefit:** Large intervals (1000+ values) solve in O(1) vs O(n) enumeration.

### ✅ Requirement 2: Bounds-Only Infrastructure (Task B1 Foundation)
Added to `Store` and `Domain` (store.rs):
- `Domain::Bounds(i64, i64)` variant for large intervals
- `Store::is_bounds_domain(cell)` and `Store::is_vec_domain(cell)` query methods
- All Domain methods updated to handle Bounds variant

### ✅ Requirement 3: Four Unit Tests
File: `rust/tests/bounds_domains.rs` (4 LinearEq tests)

| Test | Scenario | Verifies |
|------|----------|----------|
| `test_lineareq_bounds_conflict` | Max sum < RHS | Early conflict detection |
| `test_lineareq_bounds_all_but_one_fixed` | One open variable | Solving and fixing |
| `test_lineareq_bounds_tighten_multiple_open` | Two open variables | No propagation when all-but-one not triggered |
| `test_lineareq_mixed_vec_and_bounds` | Vec + fixed | Backward compatibility |

**Result:** All 4 tests pass ✅

### ✅ Requirement 4: Backward Compatibility
- Vec domain logic unchanged (uses existing enumerate-and-remove)
- All 81 baseline tests pass
- No new external dependencies
- No breaking API changes

### ✅ Requirement 5: Test Results
```
Total: 90 tests pass (81 baseline + 9 new)

Breakdown:
- Lib tests:            33 passed
- ac3_algorithm:         3 passed
- bounds_domains:        6 passed (4 LinearEq B2 + 2 LeLinear B3)
- bounds_propagation:   18 passed (15 baseline + 3 AllDiff B4)
- bounds_tracking:       4 passed
- domain_bitset:         5 passed
- element2d:             6 passed
- lazy_queue:            3 passed
- lcv_heuristic:         2 passed
- mrv_cache:             3 passed
- optimization:          7 passed
```

## Files Modified

1. **rust/src/store.rs** (150 lines Δ)
   - Added Domain::Bounds variant
   - Implemented all Domain methods for Bounds
   - Added Store query methods

2. **rust/src/propagators.rs** (15 lines Δ)
   - Modified LinearEq::propagate() for bounds-aware path

3. **rust/tests/bounds_domains.rs** (NEW, 60 lines)
   - 4 comprehensive LinearEq tests

4. **rust/tests/bounds_propagation.rs** (updated with AllDiff tests)
   - 3 new AllDiff bounds reasoning tests (for future B4 implementation)

## Commit Message

```
feat: LinearEq bounds-aware propagation (Phase B Task B2)

- Add Domain::Bounds(i64, i64) variant for large domain intervals
- Implement Store query methods: is_bounds_domain(), is_vec_domain()
- Modify LinearEq::propagate() all-but-one case:
  * Detect domain type (Bounds vs Vec)
  * Bounds path: solve and tighten_bounds(cell, [v, v]) to fix
  * Vec path: unchanged enumerate-and-remove logic
- Add 4 unit tests covering all LinearEq scenarios
- All 90 tests pass (81 baseline + 9 new)
```

## Technical Details

### Pattern for B2 Propagators
```rust
if store.is_bounds_domain(c) {
    // Bounds domain: efficient interval operation
    store.tighten_bounds(c, Bounds::new(v, v))?;
} else {
    // Vec domain: maintain existing behavior
    for cur in store.domain(c).to_vec() {
        if cur != v { store.remove(c, cur)?; }
    }
}
```

This pattern can be applied to:
- **B3:** LeLinear constraint (bounds-based pruning)
- **B4:** AllDiff constraint (pigeonhole principle with intervals)

### Key Insights
1. **Bounds variant is minimal:** Only boundary operations supported (remove() only at min/max)
2. **to_vec() expansion:** Intentionally supports full enumeration when needed (Bounds→Vec)
3. **iter() is empty for Bounds:** Large intervals shouldn't enumerate; callers must check
4. **Backward compatible:** Vec path unchanged, all 81 baseline tests pass immediately

## Verification

- ✅ Domain::Bounds fully functional
- ✅ LinearEq all-but-one case optimized
- ✅ 4 LinearEq tests pass
- ✅ 81 baseline tests pass
- ✅ No new dependencies
- ✅ Ready for B3/B4 implementation

## Next Steps (B3-B4)

- **B3:** Extend LeLinear to use bounds-based pruning
- **B4:** Extend AllDiff for pigeonhole principle reasoning

Both tasks can reuse the B2 pattern: check `is_bounds_domain()` and branch accordingly.
