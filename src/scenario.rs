use pyo3::prelude::*;
use rayon::prelude::*;

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
        match self {
            Operation::Map(f)    => Operation::Map(f.clone_ref(py)),
            Operation::Filter(f) => Operation::Filter(f.clone_ref(py)),
        }
    }
}


impl Scenario {
    fn apply_operations(&self, item: &Py<PyAny>) -> PyResult<Option<Py<PyAny>>> {
        Python::attach(|py| {
            let mut current = item.clone_ref(py);
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
        })
    }
}


#[pymethods]
impl Scenario {
    #[new]
    fn new(objects: Vec<Py<PyAny>>) -> Self {
        Scenario { objects, operations: Vec::new() }
    }

    fn map(&self, py: Python<'_>, func: Py<PyAny>) -> PyResult<Self> {
        let mut ops: Vec<Operation> = self.operations.iter()
            .map(|op| op.clone_ref(py))
            .collect();
        ops.push(Operation::Map(func));
        Ok(Scenario {
            objects: self.objects.iter()
                .map(|o| o.clone_ref(py))
                .collect(),
            operations: ops,
        })
    }

    fn filter(&self, py: Python<'_>, predicate: Py<PyAny>) -> PyResult<Self> {
        let mut ops: Vec<Operation> = self.operations.iter()
            .map(|op| op.clone_ref(py))
            .collect();
        ops.push(Operation::Filter(predicate));
        Ok(Scenario {
            objects: self.objects.iter()
                .map(|o| o.clone_ref(py))
                .collect(),
            operations: ops,
        })
    }

    #[pyo3(signature = (parallel = true))]
    fn collect(&self, py: Python<'_>, parallel: bool) -> PyResult<Vec<Py<PyAny>>> {
        if parallel {
            return py.detach(|| {
                self.objects
                    .par_iter()
                    .filter_map(|item| self.apply_operations(item).transpose())
                    .collect::<PyResult<Vec<_>>>()
            });
        } else {
            return self.objects
                .iter()
                .filter_map(|item| self.apply_operations(item).transpose())
                .collect::<PyResult<Vec<_>>>();
        }
    }
}
