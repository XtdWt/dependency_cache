use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};

use std::collections::{HashMap, HashSet};

use crate::dependency_graph::MethodDependencyGraph;
use crate::decorator::DependencyCacheDecorator;

#[pyclass(subclass)]
pub struct DependencyCacheBase {
    pub cache: HashMap<String, Py<PyAny>>,
    pub method_dependency_graph: MethodDependencyGraph,
}

impl DependencyCacheBase {
    pub fn set_cached_value(&mut self, name: &str, value: Py<PyAny>, validate: bool) {
        self.cache.insert(name.to_string(), value);
        if validate {
            self.method_dependency_graph.validate(name.to_string());
        }
    }

    fn build_dependency_graph(cls: &Bound<'_, PyType>) -> PyResult<MethodDependencyGraph> {
        let mut graph = MethodDependencyGraph::new();
        let mut visited = HashSet::new();
        let mut use_cache_methods = HashSet::new();

        for mro_class in cls.mro().iter() {
            let mro_class: Bound<'_, PyType> = mro_class.extract()?;
            let namespace = mro_class.getattr("__dict__")?;

            for item in namespace.call_method0("items")?.try_iter()? {
                let (name, value): (String, Bound<'_, PyAny>) = item?.extract()?;

                if name.starts_with("__") || !visited.insert(name.clone()) {
                    continue;
                }

                let Ok(decorator) = value.cast::<DependencyCacheDecorator>() else {
                    continue;
                };
                let decorator = decorator.borrow();
                if !decorator.use_cache {
                    use_cache_methods.insert(name.clone());
                };
                graph.add_dependency(name, decorator.dependencies.clone());
            }
        }
        let to_invalidate: Vec<String> = use_cache_methods
            .iter()
            .map(|x| graph.methods_to_invalidate(x.to_string()))
            .into_iter()
            .flatten()
            .collect::<HashSet<String>>()
            .into_iter()
            .collect();
        for method_name in to_invalidate {
            graph.cache_validation.remove(&method_name);
        };
        return Ok(graph);
    }
}

#[pymethods]
impl DependencyCacheBase {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        return Self {
            cache: HashMap::new(),
            method_dependency_graph: MethodDependencyGraph::new(),
        };
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
        return Ok(());
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
