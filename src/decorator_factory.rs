use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::exceptions::PyValueError;

use crate::decorator::DependencyCacheDecorator;
use crate::normalise_method_args::normalise_and_hash_method;


pub fn validate_self_only_method(py: Python<'_>, func: &Bound<'_, PyAny>) -> PyResult<()> {
    let inspect = py.import("inspect")?;
    let signature = inspect.call_method1("signature", (func,))?;
    let parameters = signature.getattr("parameters")?;
    let param_names: Vec<String> = parameters.call_method0("keys")?.extract()?;

    if param_names.len() != 1 || param_names[0] != "self" {
        let msg = format!(
            "Serialisable method must have exactly one parameter named 'self', got: {:?}",
            param_names
        );
        return Err(PyValueError::new_err(msg));
    }
    Ok(())
}


fn parse_dependencies(py: Python<'_>, dep_list: &Bound<'_, PyAny>) -> PyResult<Vec<(String, Py<PyDict>)>> {
    let list = dep_list.cast::<PyList>()?;
    let mut parsed = Vec::with_capacity(list.len());

    for item in list.iter() {
        if let Ok(tuple) = item.cast::<PyTuple>() {
            if tuple.len() == 2 {
                let method_name: String = tuple.get_item(0)?.extract()?;
                let kwargs = tuple.get_item(1)?;
                let kwargs_dict = kwargs.cast::<PyDict>()?;
                parsed.push((method_name, kwargs_dict.clone().unbind()));
                continue;
            }
        }

        if let Ok(method_name) = item.extract::<String>() {
            let empty_dict = PyDict::new(py);
            parsed.push((method_name, empty_dict.unbind()));
            continue;
        }

        return Err(PyValueError::new_err(
            "Dependency must be a tuple (str, dict) or a str",
        ));
    }

    return Ok(parsed);
}

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
                let (hash, _) = normalise_and_hash_method(
                    py,
                    method_name,
                    None,
                    &empty_args,
                    Some(&kwargs.bind(py)),
                )?;
                Ok(hash)
            })
            .collect::<PyResult<Vec<_>>>()?;
        validate_self_only_method(py, func.bind(py))?;
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
                let (hash, _) = normalise_and_hash_method(
                    py,
                    method_name,
                    None,
                    &empty_args,
                    Some(&kwargs.bind(py)),
                )?;
                Ok(hash)
            })
            .collect::<PyResult<Vec<_>>>()?;
        validate_self_only_method(py, func.bind(py))?;
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
