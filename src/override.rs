use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;

use crate::DependencyCacheBase;

#[pyclass]
pub struct TemporaryOverride {
    objects: Vec<Py<PyAny>>,
}


#[pymethods]
impl TemporaryOverride {
    fn __enter__<'py>(slf: &Bound<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        {
            let temporary_override = slf.borrow();
            for object in &temporary_override.objects {
                let base = object
                    .bind(py)
                    .cast::<DependencyCacheBase>()
                    .map_err(|_| {
                        PyTypeError::new_err(
                            "temporary_override() expects items to inherit from DependencyCacheBase",
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
            let temporary_override = slf.borrow();
            for objects in temporary_override.objects.iter() {
                let base = objects
                    .bind(py)
                    .cast::<DependencyCacheBase>()
                    .map_err(|_| {
                        PyTypeError::new_err(
                            "temporary_override() expects items to inherit from DependencyCacheBase",
                        )
                    })?;
                base.borrow_mut().pop_cache_stack()?;
            }
        }
        return Ok(false);
    }
}


#[pyfunction]
pub fn temporary_override(objects: Vec<Py<PyAny>>) -> PyResult<TemporaryOverride> {
    return Ok(TemporaryOverride { objects });
}
