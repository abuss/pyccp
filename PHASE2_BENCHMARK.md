## Phase 1+2 Combined Benchmark Results

**Date:** 2026-10-18  
**Branch:** v2.1-framework-upgrade  
**Optimizations Tested:**

### Phase 1 Optimizations:
- Least Constraining Value (LCV) heuristic
- MRV (Minimum Remaining Values) Cache
- Bitset domain representation

### Phase 2 Optimizations:
- Lazy propagation queue (skip unaffected constraints)
- Dependency tracking (propagator indexing)
- Bounds propagation for LinearEq and LessThan
- AC-3 algorithm for AllDiff constraint

### Results Table

| Example | Phase 1 Time | Phase 1+2 Time | Ratio | Status |
|---------|------|------|-------|--------|
| N-Queens (N=8) | 187.62 µs | 650.42 µs | 3.46x slower | ⚠ Regression |
| N-Queens (N=14) | 4.9763 ms | 18.829 ms | 3.78x slower | ⚠ Regression |
| Send+More=Money | (no baseline) | 209.82 ms | N/A | New test |

### Notes

**Performance Impact:**
The Phase 2 benchmark results show significant performance regression on N-Queens compared to Phase 1. This suggests that the current Phase 2 implementation has not yet achieved its optimization goals:

- N=8 regressed from ~187 µs to ~650 µs (3.46x slower)
- N=14 regressed from ~4.98 ms to ~18.8 ms (3.78x slower)

**Investigation Needed:**
The regression is likely due to:
1. Overhead from lazy queue management (HashMap lookups, HashSet operations)
2. Dependency tracking maintenance during domain reduction
3. Phase 2 implementation not yet fully optimized for the specific patterns in N-Queens

The lazy propagation queue should theoretically reduce wasted propagation work, but the current overhead outweighs the benefits. Further optimization of the queue implementation (e.g., using a more efficient data structure, reducing allocations) may be needed.

**Send+More=Money Baseline:**
This cryptarithmetic problem is included as a new baseline for Phase 2 testing (209.82 ms median). No Phase 1 equivalent was previously measured, so this will serve as the reference point for measuring improvements.

**Next Steps:**
- Profile the lazy queue overhead on N-Queens to identify bottlenecks
- Consider using a more efficient dependency tracking structure (e.g., Vec-based index instead of HashMap)
- Optimize the propagators_for_vars() lookup to reduce allocations
- Verify that lazy queue benefits larger problems with fewer propagator dependencies

**Full before/after (v0.2.0 vs v2.1):**
A complete baseline comparison will be done after merging to master and comparing against the main branch baseline.

### Raw Benchmark Output

See `PHASE2_BENCHMARK.log` for detailed criterion output including outlier analysis and statistical confidence intervals.
