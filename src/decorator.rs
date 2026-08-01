use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use crate::dependency_cache_base::DependencyCacheBase;

#[pyclass(frozen)]
pub struct DependencyCacheDecorator {
    pub func: Py<PyAny>,
    pub use_cache: bool,
    pub dependencies: Vec<String>,
    pub method_name: String,
    pub track_runtime_dependencies: bool,
}

#[pymethods]
impl DependencyCacheDecorator {
    fn __set__(&self, _obj: Py<PyAny>, _value: Py<PyAny>) -> PyResult<()> {
        return Err(PyTypeError::new_err("cannot assign to decorated method"));
    }

    fn __get__(
        slf: &Bound<'_, Self>,
        obj: Bound<'_, PyAny>,
        _objtype: Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        let py = slf.py();

        if obj.is_none() {
            return Ok(slf.clone().into_any().unbind());
        }

        let bound_method = py
            .import("types")?
            .getattr("MethodType")?
            .call1((slf.clone(), obj))?;
        return Ok(bound_method.unbind());
    }

    #[pyo3(signature = (*args, **kwargs))]
    fn __call__(
        &self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        if args.is_empty() {
            return Err(PyValueError::new_err(
                "dependency_cached can only be used as an instance method decorator",
            ));
        }

        let instance = args.get_item(0)?;
        let base = instance.cast::<DependencyCacheBase>().map_err(|_| {
            PyTypeError::new_err(
                "the decorated method's class must inherit from DependencyCacheBase",
            )
        })?;

        let func = self.func.bind(py);

        // 1. add/check child dependencies
        base.borrow_mut().add_children_dependencies(&self.method_name, self.dependencies.clone());

        // 2. add/check parent dependencies, push function stack
        let parent_status = base.borrow().current_call_stack_top();
        if let Some((parent, parent_use_runtime_deps)) = parent_status {
            if parent_use_runtime_deps {
                base.borrow_mut().add_parent_dependency(&parent, &self.method_name);
            }
        }
        base.borrow_mut().push_call_stack(self.method_name.clone(), self.track_runtime_dependencies);

        let outcome = (|| -> PyResult<Py<PyAny>> {
            // 3. check cache for value
            let cached = base.borrow().get_cached_value(py, &self.method_name);
            if let Some(cached) = cached {
                return Ok(cached);
            }

            // 4. run function
            let result = func.call(args, kwargs)?.unbind();

            // 5. validate current, set cache and pop function stack
            base.borrow_mut()
                .validate_current_method(&self.method_name, self.use_cache);
            base.borrow_mut()
                .set_cached_value(&self.method_name, result.clone_ref(py));
            Ok(result)
        })();

        base.borrow_mut().pop_call_stack();

        return outcome;
    }
}
