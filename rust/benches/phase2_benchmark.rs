/// Phase 2 Benchmark Suite
/// Measures performance improvements from lazy queue + dependency tracking optimizations
/// Built on top of Phase 1 (LCV + MRV Cache + Bitset)
/// Benchmarks N-Queens with N=8 and N=14, plus Send+More=Money cryptarithmetic

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

/// Helper: set up Send+More=Money cryptarithmetic problem
fn setup_send_more_money() -> CustomSolver {
    // 8 variables: S E N D M O R Y
    let domains: Vec<Vec<i64>> = (0..8).map(|_| (0..10).collect()).collect();
    let mut solver = CustomSolver::new(&domains);

    // All different
    solver.post(Constraint::AllDiff(vec![0, 1, 2, 3, 4, 5, 6, 7])).unwrap();
    
    // S and M are non-zero (leading digits)
    solver.post(Constraint::NonZero(0)).unwrap();
    solver.post(Constraint::NonZero(4)).unwrap();

    // SEND + MORE = MONEY
    // 1000*S + 100*E + 10*N + D + 1000*M + 100*O + 10*R + E = 10000*M + 1000*O + 100*N + 10*E + Y
    // Simplified: S*1000 + E*100 + N*10 + D + M*1000 + O*100 + R*10 + E - M*10000 - O*1000 - N*100 - E*10 - Y = 0
    // Reordered: S*1000 + E*(100+1-10) + N*(10-100) + D + M*(1000-10000) + O*(100-1000) + R*10 + Y*(-1) = 0
    solver
        .post(Constraint::LinearEq(
            vec![
                (0, 1000),   // S
                (1, 91),     // E: 100 + 1 - 10
                (2, -90),    // N: 10 - 100
                (3, 1),      // D
                (4, -9000),  // M: 1000 - 10000
                (5, -900),   // O: 100 - 1000
                (6, 10),     // R
                (7, -1),     // Y
            ],
            0,
        ))
        .unwrap();

    solver
}

/// Benchmark N-Queens with N=8 (Phase 2)
fn benchmark_nqueens_8_phase2(c: &mut Criterion) {
    c.bench_function("nqueens_8_phase2", |b| {
        b.iter(|| {
            let mut solver = setup_nqueens(black_box(8));
            solver.solve_parallel()
        })
    });
}

/// Benchmark N-Queens with N=14 (Phase 2)
fn benchmark_nqueens_14_phase2(c: &mut Criterion) {
    c.bench_function("nqueens_14_phase2", |b| {
        b.iter(|| {
            let mut solver = setup_nqueens(black_box(14));
            solver.solve_parallel()
        })
    });
}

/// Benchmark Send+More=Money cryptarithmetic (Phase 2)
fn benchmark_send_more_money_phase2(c: &mut Criterion) {
    c.bench_function("send_more_money_phase2", |b| {
        b.iter(|| {
            let mut solver = setup_send_more_money();
            solver.solve_parallel()
        })
    });
}

criterion_group!(
    benches,
    benchmark_nqueens_8_phase2,
    benchmark_nqueens_14_phase2,
    benchmark_send_more_money_phase2
);
criterion_main!(benches);
