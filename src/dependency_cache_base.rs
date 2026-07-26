use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};

use std::collections::HashMap;

use crate::dependency_graph::MethodDependencyGraph;
use crate::decorator::DependencyCacheDecorator;

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

    fn build_dependency_graph(cls: &Bound<'_, PyType>) -> PyResult<MethodDependencyGraph> {
            let mut graph = MethodDependencyGraph::new();
            let mut seen = std::collections::HashSet::new();

            for klass in cls.mro().iter() {
                let klass: Bound<'_, PyType> = klass.extract()?;
                let namespace = klass.getattr("__dict__")?;

                for item in namespace.call_method0("items")?.try_iter()? {
                    let (name, value): (String, Bound<'_, PyAny>) = item?.extract()?;

                    if name.starts_with("__") || !seen.insert(name.clone()) {
                        continue;
                    }

                    // Only attributes that are genuinely DependencyCacheDecorator
                    // instances participate in the graph; plain methods are
                    // skipped rather than erroring.
                    let Ok(decorator) = value.cast::<DependencyCacheDecorator>() else {
                        continue;
                    };
                    let decorator = decorator.borrow();

                    graph.add_dependency(name, decorator.dependencies.clone());
                }
            }

            Ok(graph)
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

    #[pyo3(signature = (*_args, **_kwargs))]
    fn __init__(
        slf: &Bound<'_, Self>,
        _args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let cls = slf.get_type();
        let graph = Self::build_dependency_graph(&cls)?;
        slf.borrow_mut().method_dependency_graph = graph;
        Ok(())
    }

    pub fn get_cached_value(&self, py: Python<'_>, name: &str) -> Option<Py<PyAny>> {
        if self.method_dependency_graph.is_valid(name.to_string()) {
            return self.cache.get(name).map(|obj| obj.clone_ref(py));
        }
        return None;
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
        return Ok(dict);
    }

    pub fn current_graph<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.method_dependency_graph.cache_dependency_graph {
            dict.set_item(name, value)?;
        }
        return Ok(dict);
    }

    pub fn current_cache_validation<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.method_dependency_graph.cache_validation {
            dict.set_item(name, value)?;
        }
        return Ok(dict);
    }
}
