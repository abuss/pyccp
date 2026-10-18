## Phase 1 Benchmark Results (LCV + MRV Cache + Bitset)

**Date:** 2026-10-18  
**Branch:** v2.1-framework-upgrade  
**Optimizations Tested:**
- Least Constraining Value (LCV) heuristic
- MRV (Minimum Remaining Values) Cache
- Bitset domain representation

### Results Table

| Example | Time | Status |
|---------|------|--------|
| N-Queens (N=8) | 185.56 µs - 189.86 µs | ✓ PASS |
| N-Queens (N=14) | 4.9372 ms - 5.0183 ms | ✓ PASS |

**Notes:**
- N=8 solved in ~187 µs (median)
- N=14 solved in ~4.96 ms (median)
- All benchmarks completed without errors
- Criterion collected 100 samples per benchmark with <4% outliers

**Baseline Status:**
This is the Phase 1 v2.1-framework-upgrade version (no baseline available yet on this branch).  
Full before/after comparison will be done after merging to master and comparing against the main branch baseline.

### Raw Benchmark Output

See `PHASE1_BENCHMARK.log` for detailed criterion output including outlier analysis and statistical confidence intervals.
