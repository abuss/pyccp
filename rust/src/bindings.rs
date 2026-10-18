use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;

use crate::solver::{Constraint, CustomSolver, Solver};
use crate::store::Conflict;

create_exception!(pyccp_core, ConflictError, PyException);

fn to_pyerr(_: Conflict) -> PyErr { ConflictError::new_err("constraint conflict") }

fn parse_constraint(c: &Bound<'_, PyAny>, n: usize) -> PyResult<Constraint> {
    let check = |i: usize| -> PyResult<usize> {
        if i >= n { return Err(PyValueError::new_err(format!("cell {i} out of bounds"))); }
        Ok(i)
    };
    let kind: String = c.get_item(0)?.extract()?;
    let args = c.get_item(1)?;
    match kind.as_str() {
        "neq" => {
            let i = check(args.get_item(0)?.extract()?)?;
            let j = check(args.get_item(1)?.extract()?)?;
            Ok(Constraint::NeqLinear(vec![(i, 1), (j, -1)], 0))
        }
        "all_diff" => {
            let cells: Vec<usize> = args.get_item(0)?.extract()?;
            for &i in &cells { check(i)?; }
            Ok(Constraint::AllDiff(cells))
        }
        "nonzero" => Ok(Constraint::NonZero(check(args.get_item(0)?.extract()?)?)),
        "linear_eq" => {
            let terms: Vec<(usize, i64)> = args.get_item(0)?.extract()?;
            for &(i, _) in &terms { check(i)?; }
            let rhs = args.get_item(1)?.extract()?;
            Ok(Constraint::LinearEq(terms, rhs))
        }
        "lt" => Ok(Constraint::LessThan(check(args.get_item(0)?.extract()?)?, check(args.get_item(1)?.extract()?)?)),
        "mul" => Ok(Constraint::Multiply(
            check(args.get_item(0)?.extract()?)?, check(args.get_item(1)?.extract()?)?, args.get_item(2)?.extract()?)),
        "element" => {
            let array: Vec<usize> = args.get_item(0)?.extract()?;
            for &i in &array { check(i)?; }
            Ok(Constraint::Element(array, check(args.get_item(1)?.extract()?)?, check(args.get_item(2)?.extract()?)?))
        }
        "cardinality" => {
            let cells: Vec<usize> = args.get_item(0)?.extract()?;
            for &i in &cells { check(i)?; }
            Ok(Constraint::Cardinality(cells, args.get_item(1)?.extract()?, args.get_item(2)?.extract()?))
        }
        "modulo" => Ok(Constraint::Modulo(
            check(args.get_item(0)?.extract()?)?, args.get_item(1)?.extract()?, args.get_item(2)?.extract()?)),
        "and" | "or" | "xor" => {
            let kind = kind.clone();
            let x = check(args.get_item(0)?.extract()?)?;
            let y = check(args.get_item(1)?.extract()?)?;
            let z = check(args.get_item(2)?.extract()?)?;
            Ok(match kind.as_str() {
                "and" => Constraint::And(x, y, z),
                "or"  => Constraint::Or(x, y, z),
                _     => Constraint::Xor(x, y, z),
            })
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

    fn solve_all(&mut self, limit: usize) -> Vec<Vec<i64>> {
        self.solver.solve_all(limit)
    }

    fn post(&mut self, py: Python, c: &Bound<'_, PyAny>) -> PyResult<()> {
        let c = parse_constraint(c, self.fixed_before.len())?;
        self.solver.post(c).map_err(to_pyerr)?;
        self.notify_newly_fixed(py)?;
        Ok(())
    }

    fn commit(&mut self, py: Python, cell: usize, value: i64) -> PyResult<()> {
        if cell >= self.fixed_before.len() {
            return Err(PyValueError::new_err(format!("cell {cell} out of bounds")));
        }
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
