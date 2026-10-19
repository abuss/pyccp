/// Phase 1 Benchmark Suite
/// Measures performance improvements from LCV + MRV Cache + Bitset optimizations
/// Benchmarks N-Queens with N=8 and N=14

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyccp_core::solver::{Constraint, CustomSolver, Solver};

/// Helper: set up N-Queens problem with n queens
fn setup_nqueens(n: usize) -> CustomSolver {
    let domains: Vec<Vec<i64>> = (0..n).map(|_| (0..n as i64).collect()).collect();
    let mut solver = CustomSolver::new(&domains);

    // Add constraints: all queens must be in different rows (columns) and diagonals
    for i in 0..n {
        for j in (i + 1)..n {
            // Column constraints: column[i] != column[j]
            solver
                .post(Constraint::NeqLinear(vec![(i, 1), (j, -1)], 0))
                .unwrap();
            // Diagonal constraints: column[i] - i != column[j] - j
            solver
                .post(Constraint::NeqLinear(
                    vec![(i, 1), (j, -1)],
                    i as i64 - j as i64,
                ))
                .unwrap();
            // Anti-diagonal constraints: column[i] + i != column[j] + j
            solver
                .post(Constraint::NeqLinear(
                    vec![(i, 1), (j, -1)],
                    j as i64 - i as i64,
                ))
                .unwrap();
        }
    }

    solver
}

/// Benchmark N-Queens with N=8
fn benchmark_nqueens_8(c: &mut Criterion) {
    c.bench_function("nqueens_8", |b| {
        b.iter(|| {
            let mut solver = setup_nqueens(black_box(8));
            solver.solve()
        })
    });
}

/// Benchmark N-Queens with N=14
fn benchmark_nqueens_14(c: &mut Criterion) {
    c.bench_function("nqueens_14", |b| {
        b.iter(|| {
            let mut solver = setup_nqueens(black_box(14));
            solver.solve()
        })
    });
}

criterion_group!(benches, benchmark_nqueens_8, benchmark_nqueens_14);
criterion_main!(benches);
