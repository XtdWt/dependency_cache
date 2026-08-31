use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use crate::decorator::DependencyCacheDecorator;
use crate::py_introspection_utils::{normalise_function_signature_and_hash, parse_dependencies, validate_self_only_method};


#[pyclass(name = "dependency_cached", frozen)]
pub struct ManualDependencyCacheDecoratorFactory {
    use_cache: bool,
    dependencies: Vec<(String, Py<PyDict>)>,
    track_runtime_dependencies: bool,
    serialisable: bool,
}

#[pymethods]
impl ManualDependencyCacheDecoratorFactory {

    #[new]
    #[pyo3(signature = (use_cache=true, dependencies=None, track_runtime_dependencies=false, serialisable=false))]
    fn new(
        py: Python<'_>,
        use_cache: bool,
        dependencies: Option<&Bound<'_, PyAny>>,
        track_runtime_dependencies: bool,
        serialisable: bool,
    ) -> PyResult<Self> {
        let parsed_deps = if let Some(list) = dependencies {
            parse_dependencies(py, list)?
        } else {
            Vec::new()
        };
        return Ok(Self {
            use_cache,
            dependencies: parsed_deps,
            track_runtime_dependencies,
            serialisable,
        });
    }

    fn __call__(&self, py: Python<'_>, func: Py<PyAny>) -> PyResult<DependencyCacheDecorator> {
        let method_name: String = func.getattr(py, "__name__")?.extract(py)?;
        let empty_args = PyTuple::empty(py);
        let hashed_dependencies: Vec<isize> = self
            .dependencies
            .iter()
            .map(|(method_name, kwargs)| {
                let (hash, _) = normalise_function_signature_and_hash(
                    py,
                    method_name,
                    None,
                    &empty_args,
                    Some(&kwargs.bind(py)),
                )?;
                Ok(hash)
            })
            .collect::<PyResult<Vec<_>>>()?;
        if self.serialisable {
            validate_self_only_method(py, func.bind(py))?;
        }
        return Ok(DependencyCacheDecorator {
            func,
            use_cache: self.use_cache,
            dependencies: hashed_dependencies,
            method_name,
            track_runtime_dependencies: self.track_runtime_dependencies,
            serialisable: self.serialisable,
        });
    }
}

#[pyclass(name = "automagically_dependency_cached", frozen)]
pub struct AutomagicDependencyCacheDecoratorFactory {
    use_cache: bool,
    dependencies: Vec<(String, Py<PyDict>)>,
    track_runtime_dependencies: bool,
    serialisable: bool,
}

#[pymethods]
impl AutomagicDependencyCacheDecoratorFactory {
    #[new]
    #[pyo3(signature = (use_cache=true, dependencies=None, track_runtime_dependencies=true, serialisable=false))]
    fn new(
        py: Python<'_>,
        use_cache: bool,
        dependencies: Option<&Bound<'_, PyAny>>,
        track_runtime_dependencies: bool,
        serialisable: bool,
    ) -> PyResult<Self> {
        let parsed_deps = if let Some(list) = dependencies {
            parse_dependencies(py, list)?
        } else {
            Vec::new()
        };
        return Ok(Self {
            use_cache,
            dependencies: parsed_deps,
            track_runtime_dependencies,
            serialisable,
        });
    }

    fn __call__(&self, py: Python<'_>, func: Py<PyAny>) -> PyResult<DependencyCacheDecorator> {
        let method_name: String = func.getattr(py, "__name__")?.extract(py)?;
        let empty_args = PyTuple::empty(py);
        let hashed_dependencies: Vec<isize> = self
            .dependencies
            .iter()
            .map(|(method_name, kwargs)| {
                let (hash, _) = normalise_function_signature_and_hash(
                    py,
                    method_name,
                    None,
                    &empty_args,
                    Some(&kwargs.bind(py)),
                )?;
                Ok(hash)
            })
            .collect::<PyResult<Vec<_>>>()?;
        if self.serialisable {
            validate_self_only_method(py, func.bind(py))?;
        }
        return Ok(DependencyCacheDecorator {
            func,
            use_cache: self.use_cache,
            dependencies: hashed_dependencies,
            method_name,
            track_runtime_dependencies: self.track_runtime_dependencies,
            serialisable: self.serialisable,
        });
    }
}
