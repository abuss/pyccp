use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;

use crate::solver::{Constraint, CustomSolver, Solver};
use crate::store::Conflict;

create_exception!(pyccp_core, ConflictError, PyException);

fn to_pyerr(c: Conflict) -> PyErr { ConflictError::new_err("constraint conflict") }

fn parse_constraint(py: Python, c: &Bound<'_, PyAny>) -> PyResult<Constraint> {
    let kind: String = c.get_item(0)?.extract()?;
    let args = c.get_item(1)?;
    match kind.as_str() {
        "neq" => {
            let i: usize = args.get_item(0)?.extract()?;
            let j: usize = args.get_item(1)?.extract()?;
            Ok(Constraint::NeqLinear(vec![(i, 1), (j, -1)], 0))
        }
        "all_diff" => Ok(Constraint::AllDiff(args.get_item(0)?.extract()?)),
        "nonzero" => Ok(Constraint::NonZero(args.get_item(0)?.extract()?)),
        "linear_eq" => {
            let terms: Vec<(usize, i64)> = args.get_item(0)?.extract()?;
            let rhs = args.get_item(1)?.extract()?;
            Ok(Constraint::LinearEq(terms, rhs))
        }
        other => Err(PyValueError::new_err(format!("unknown kind: {other}"))),
    }
}

#[pyclass(name = "Store")]
pub struct PyStore {
    solver: CustomSolver,
}

#[pymethods]
impl PyStore {
    #[new]
    fn new(domains: Vec<Vec<i64>>) -> PyStore {
        PyStore { solver: CustomSolver::new(&domains) }
    }

    fn post(&mut self, py: Python, c: &Bound<'_, PyAny>) -> PyResult<()> {
        let c = parse_constraint(py, c)?;
        self.solver.post(c).map_err(to_pyerr)
    }

    fn solve(&mut self) -> Option<Vec<i64>> {
        self.solver.solve()
    }
}

#[pymodule]
fn pyccp_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyStore>()?;
    m.add("Conflict", m.py().get_type::<ConflictError>())?;
    Ok(())
}
