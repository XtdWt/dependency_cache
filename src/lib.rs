mod decorator;
mod dependency_graph;
mod dependency_cache_base;
mod decorator_factory;
mod py_introspection_utils;
mod metadata_hash_manager;
mod r#override;
mod scenario;

use pyo3::prelude::*;

use dependency_cache_base::DependencyCacheBase;
use decorator_factory::{ManualDependencyCacheDecoratorFactory, AutomagicDependencyCacheDecoratorFactory};
use r#override::temporary_override;
use scenario::Scenario;
use decorator::read_items;


use std::sync::atomic::{AtomicU64, Ordering};

use pyo3::exceptions::{PyAttributeError, PyRuntimeError};
// use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use pyo3::ffi;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[pyclass(frozen)]
struct CopyLocal {
    /// Key under which this instance's dict lives in each thread's state dict.
    key: Py<PyString>,
    /// The creating thread's dict. New threads start from a copy of it.
    parent: Py<PyDict>,
}


impl CopyLocal {
    fn thread_state_dict<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let ptr = unsafe { ffi::PyThreadState_GetDict() };
        if ptr.is_null() {
            return Err(PyRuntimeError::new_err("no thread state dict"));
        }
        let obj = unsafe { Bound::from_borrowed_ptr(py, ptr) };
        Ok(obj.cast_into::<PyDict>()?)
    }

    fn local_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let ts = Self::thread_state_dict(py)?;
        let key = self.key.bind(py);

        if let Some(d) = ts.get_item(key)? {
            return Ok(d.cast_into::<PyDict>()?);
        }
        let d = self.parent.bind(py).copy()?;
        ts.set_item(key, &d)?;
        Ok(d)
    }
}


#[pymethods]
impl CopyLocal {
    #[new]
    fn new(py: Python<'_>) -> PyResult<Self> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let key = PyString::new(py, &format!("_copylocal_{id}")).unbind();

        // The creating thread's dict is the copy source.
        let parent = PyDict::new(py);
        Self::thread_state_dict(py)?.set_item(key.bind(py), &parent)?;

        Ok(Self { key, parent: parent.unbind() })
    }

    fn __getattr__<'py>(
        &self,
        py: Python<'py>,
        name: Bound<'py, PyString>,
    ) -> PyResult<Bound<'py, PyAny>> {
        match self.local_dict(py)?.get_item(&name)? {
            Some(v) => Ok(v),
            None => Err(PyAttributeError::new_err(format!(
                "'CopyLocal' object has no attribute '{name}'"
            ))),
        }
    }

    fn __setattr__(
        &self,
        py: Python<'_>,
        name: Bound<'_, PyString>,
        value: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.local_dict(py)?.set_item(name, value)
    }

    fn __delattr__(&self, py: Python<'_>, name: Bound<'_, PyString>) -> PyResult<()> {
        let d = self.local_dict(py)?;
        if !d.contains(&name)? {
            return Err(PyAttributeError::new_err(format!("{name}")));
        }
        d.del_item(name)
    }

    fn hello(&self, py: Python<'_>) -> Py<PyString> {
        self.key.clone_ref(py)
    }
}


#[pymodule]
mod dependency_cache {

    #[pymodule_export]
    use super::DependencyCacheBase;

    #[pymodule_export]
    use super::ManualDependencyCacheDecoratorFactory;

    #[pymodule_export]
    use super::AutomagicDependencyCacheDecoratorFactory;

    #[pymodule_export]
    use super::temporary_override;

    #[pymodule_export]
    use super::Scenario;

    #[pymodule_export]
    use super::read_items;

    #[pymodule_export]
    use super::CopyLocal;
}
