use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use rayon::prelude::*;

use crate::DependencyCacheBase;

#[pyclass]
pub struct Scenario {
    objects: Vec<Py<PyAny>>,
    operations: Vec<Operation>,
}

enum Operation {
    Map(Py<PyAny>),
    Filter(Py<PyAny>),
}

impl Operation {
    fn clone_ref(&self, py: Python<'_>) -> Self {
        return match self {
            Operation::Map(f)    => Operation::Map(f.clone_ref(py)),
            Operation::Filter(f) => Operation::Filter(f.clone_ref(py)),
        };
    }
}


impl Scenario {
    fn apply_operations(&self, object: &Py<PyAny>) -> PyResult<Option<Py<PyAny>>> {
        return Python::attach(|py| {
            let mut current = object.clone_ref(py);
            for op in &self.operations {
                match op {
                    Operation::Map(func) => {
                        current = func.call1(py, (current,))?;
                    }
                    Operation::Filter(cond) => {
                        let keep: bool = cond
                            .call1(py, (current.clone_ref(py),))?
                            .extract(py)?;
                        if !keep {
                            return Ok(None);
                        }
                    }
                }
            }
            Ok(Some(current))
        });
    }

    fn push_overrides(&self, py: Python<'_>) -> PyResult<()> {
        for item in &self.objects {
            let base = item.bind(py).cast::<DependencyCacheBase>()?;
            base.borrow_mut().push_cache_stack();
        }
        return Ok(());
    }

    fn pop_overrides(&self, py: Python<'_>) -> Result<(), PyErr> {
        let mut first_err: Option<PyErr> = None;
        for item in &self.objects {
            let base = item.bind(py).cast::<DependencyCacheBase>()?;
            if let Err(e) = base.borrow_mut().pop_cache_stack() {
                first_err.get_or_insert(e);
            }
        }
        return first_err.map_or(Ok(()), Err);
    }
}


#[pymethods]
impl Scenario {
    #[new]
    fn new(py: Python<'_>, objects: Vec<Py<PyAny>>) -> PyResult<Self> {
        for object in &objects {
            object.bind(py).cast::<DependencyCacheBase>().map_err(|_| {
                PyTypeError::new_err(
                    "Scenario items must inherit from DependencyCacheBase",
                )
            })?;
        }
        return Ok(Scenario { objects, operations: Vec::new() });
    }

    fn map(&self, py: Python<'_>, func: Py<PyAny>) -> PyResult<Self> {
        let mut ops: Vec<Operation> = self.operations.iter()
            .map(|op| op.clone_ref(py))
            .collect();
        ops.push(Operation::Map(func));
        return Ok(Scenario {
            objects: self.objects.iter()
                .map(|o| o.clone_ref(py))
                .collect(),
            operations: ops,
        });
    }

    fn filter(&self, py: Python<'_>, predicate: Py<PyAny>) -> PyResult<Self> {
        let mut ops: Vec<Operation> = self.operations.iter()
            .map(|op| op.clone_ref(py))
            .collect();
        ops.push(Operation::Filter(predicate));
        return Ok(Scenario {
            objects: self.objects.iter()
                .map(|o| o.clone_ref(py))
                .collect(),
            operations: ops,
        });
    }

    #[pyo3(signature = (parallel = true))]
    fn collect(&self, py: Python<'_>, parallel: bool) -> PyResult<Vec<Py<PyAny>>> {
        self.push_overrides(py)?;
        let results = if parallel {
            py.detach(|| {
                self.objects
                    .par_iter()
                    .filter_map(|object| self.apply_operations(object).transpose())
                    .collect::<PyResult<Vec<_>>>()
            })
        } else {
            self.objects
                .iter()
                .filter_map(|object| self.apply_operations(object).transpose())
                .collect::<PyResult<Vec<_>>>()
        };
        self.pop_overrides(py)?;
        return results;
    }
}
