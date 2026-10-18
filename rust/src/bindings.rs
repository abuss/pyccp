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
    queue: Py<PyAny>,
    loop_: Py<PyAny>,
    wake: Py<PyAny>,
    fixed_before: Vec<bool>,
}

#[pymethods]
impl PyStore {
    #[new]
    fn new(domains: Vec<Vec<i64>>, queue: Py<PyAny>, loop_: Py<PyAny>, wake: Py<PyAny>) -> PyStore {
        let n = domains.len();
        PyStore {
            solver: CustomSolver::new(&domains),
            queue, loop_, wake,
            fixed_before: vec![false; n],
        }
    }

    fn post(&mut self, py: Python, c: &Bound<'_, PyAny>) -> PyResult<()> {
        let c = parse_constraint(py, c)?;
        self.solver.post(c).map_err(to_pyerr)?;
        self.notify_newly_fixed(py)?;
        Ok(())
    }

    fn commit(&mut self, py: Python, cell: usize, value: i64) -> PyResult<()> {
        self.solver.commit(cell, value).map_err(to_pyerr)?;
        self.notify_newly_fixed(py)?;
        Ok(())
    }

    fn solve(&mut self, py: Python) -> Option<Vec<i64>> {
        let sol = self.solver.solve();
        // main branch only: report final fixed cells
        for (cell, &v) in sol.iter().flatten().enumerate() {
            if !self.fixed_before[cell] {
                self.notify(py, cell, Some(v)).ok();
            }
        }
        sol
    }
}

impl PyStore {
    fn notify(&self, py: Python, cell: usize, value: Option<i64>) -> PyResult<()> {
        if self.loop_.bind(py).is_none() { return Ok(()); }
        let fact = (cell, value).into_pyobject(py)?;
        self.queue.bind(py).call_method1("put", (fact,))?;
        let set = self.wake.bind(py).getattr("set")?;
        self.loop_.bind(py).call_method1("call_soon_threadsafe", (set,))?;
        Ok(())
    }

    fn notify_newly_fixed(&mut self, py: Python) -> PyResult<()> {
        for cell in 0..self.fixed_before.len() {
            if !self.fixed_before[cell] && self.solver.is_fixed(cell) {
                self.fixed_before[cell] = true;
                self.notify(py, cell, Some(self.solver.value(cell)))?;
            }
        }
        Ok(())
    }
}

#[pymodule]
fn pyccp_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyStore>()?;
    m.add("Conflict", m.py().get_type::<ConflictError>())?;
    Ok(())
}
