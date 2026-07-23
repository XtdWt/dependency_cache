use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use std::collections::HashMap;

use crate::dependency_graph::MethodDependencyGraph;


#[pyclass(subclass)]
pub struct DependencyCacheBase {
    pub cache: HashMap<String, Py<PyAny>>,
    pub method_dependency_graph: MethodDependencyGraph,
}


impl DependencyCacheBase {
    pub fn set_cached_value(&mut self, name: &str, value: Py<PyAny>) {
        self.cache.insert(name.to_string(), value);
        self.method_dependency_graph.validate(name.to_string());
    }
}

#[pymethods]
impl DependencyCacheBase {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        Self {
            cache: HashMap::new(),
            method_dependency_graph: MethodDependencyGraph::new(),
        }
    }

    pub fn get_cached_value(&self, py: Python<'_>, name: &str) -> Option<Py<PyAny>> {
        if self.method_dependency_graph.is_valid(name.to_string()) {
            return self.cache.get(name).map(|obj| obj.clone_ref(py));
        }
        None
    }

    pub fn update_cached_value(&mut self, name: &str, value: Py<PyAny>) {
        self.method_dependency_graph.invalidate(name.to_string());
        self.cache.insert(name.to_string(), value);
        self.method_dependency_graph.validate(name.to_string());
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn current_cache<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.cache {
            dict.set_item(name, value.clone_ref(py))?;
        }
        Ok(dict)
    }

    pub fn current_graph<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.method_dependency_graph.cache_dependency_graph {
            dict.set_item(name, value)?;
        }
        Ok(dict)
    }

    pub fn current_cache_validation<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.method_dependency_graph.cache_validation {
            dict.set_item(name, value)?;
        }
        Ok(dict)
    }
}
