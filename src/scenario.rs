use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;

use crate::DependencyCacheBase;

#[pyclass]
pub struct Scenario {
    items: Vec<Py<PyAny>>,
}


#[pymethods]
impl Scenario {
    fn __enter__<'py>(slf: &Bound<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        {
            let scenario = slf.borrow();
            for item in &scenario.items {
                let base = item
                    .bind(py)
                    .cast::<DependencyCacheBase>()
                    .map_err(|_| {
                        PyTypeError::new_err(
                            "custom_scenario() expects items to inherit from DependencyCacheBase",
                        )
                    })?;
                base.borrow_mut().push_cache_stack();
            }
        }
        return Ok(slf.clone().into_any());
    }

    fn __exit__<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
        _exc_type: Option<Bound<'_, PyAny>>,
        _exc_value: Option<Bound<'_, PyAny>>,
        _traceback: Option<Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        {
            let scenario = slf.borrow();
            for item in scenario.items.iter() {
                let base = item
                    .bind(py)
                    .cast::<DependencyCacheBase>()
                    .map_err(|_| {
                        PyTypeError::new_err(
                            "custom_scenario() expects items to inherit from DependencyCacheBase",
                        )
                    })?;
                base.borrow_mut().pop_cache_stack()?;
            }
        }
        return Ok(false);
    }
}


#[pyfunction]
pub fn custom_scenario(items: Vec<Py<PyAny>>) -> PyResult<Scenario> {
    return Ok(Scenario { items });
}
