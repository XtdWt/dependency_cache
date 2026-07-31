use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use std::collections::{HashMap};

use crate::dependency_graph::{MethodDependencyGraph, ValidationState};

#[pyclass(subclass)]
pub struct DependencyCacheBase {
    pub cache: HashMap<String, Py<PyAny>>,
    pub method_dependency_graph: MethodDependencyGraph,
    pub call_stack: Vec<(String, bool)>,
}

impl DependencyCacheBase {
    pub fn set_cached_value(&mut self, name: &str, value: Py<PyAny>) {
        if !self.method_dependency_graph.is_valid(name.to_string()) {
            return ();
        }
        self.cache.insert(name.to_string(), value);
    }

    pub fn validate_current_method(&mut self, method: &str, use_cache: bool) {
        if !use_cache {
            self.method_dependency_graph.permanently_invalidate(method.to_string());
        }

        let child_validation_states: Vec<String> = self.method_dependency_graph.list_child_methods(method);

        let mut new_state = ValidationState::Valid;

            for child_state in &child_validation_states {
                match self.method_dependency_graph.get_method_state_as_enum(child_state) {
                    ValidationState::PermanentlyInvalid => {
                        new_state = ValidationState::PermanentlyInvalid;
                        break;
                    }
                    ValidationState::Invalid => {new_state = ValidationState::Invalid;}
                    ValidationState::Valid => {}
                }
            }

            match new_state {
                ValidationState::PermanentlyInvalid => {
                    self.method_dependency_graph.permanently_invalidate(method.to_string());
                }
                ValidationState::Invalid => {
                    self.method_dependency_graph.temporarily_invalidate(method.to_string());
                }
                ValidationState::Valid => {
                    self.method_dependency_graph.validate(method.to_string());
                }
            }
        }

    pub fn current_call_stack_top(&self) -> Option<(String, bool)> {
        self.call_stack.last().cloned()
    }

    pub fn push_call_stack(&mut self, name: String, add_parent_dependencies: bool) {
        self.call_stack.push((name, add_parent_dependencies));
    }

    pub fn pop_call_stack(&mut self) {
        self.call_stack.pop();
    }

    pub fn add_parent_dependency(&mut self, parent: &str, dependency: &str) {
        self.method_dependency_graph
            .add_parent_dependency(parent.to_string(), vec![dependency.to_string()]);
    }

    pub fn add_children_dependencies(&mut self, parent: &str, dependency: Vec<String>) {
        self.method_dependency_graph
            .add_children_dependency(parent.to_string(),dependency);
    }

    // fn build_dependency_graph(cls: &Bound<'_, PyType>) -> PyResult<MethodDependencyGraph> {
    //     let mut graph = MethodDependencyGraph::new();
    //     let mut visited = HashSet::new();
    //     let mut use_cache_methods = HashSet::new();

    //     for mro_class in cls.mro().iter() {
    //         let mro_class: Bound<'_, PyType> = mro_class.extract()?;
    //         let namespace = mro_class.getattr("__dict__")?;

    //         for item in namespace.call_method0("items")?.try_iter()? {
    //             let (name, value): (String, Bound<'_, PyAny>) = item?.extract()?;

    //             if name.starts_with("__") || !visited.insert(name.clone()) {
    //                 continue;
    //             }

    //             let Ok(decorator) = value.cast::<DependencyCacheDecorator>() else {
    //                 continue;
    //             };
    //             let decorator = decorator.borrow();
    //             if !decorator.use_cache {
    //                 use_cache_methods.insert(name.clone());
    //             };
    //             graph.add_dependency(name, decorator.dependencies.clone());
    //         }
    //     }
    //     let to_invalidate: Vec<String> = use_cache_methods
    //         .iter()
    //         .flat_map(|x| graph.methods_to_invalidate(x.to_string()))
    //         .collect::<HashSet<String>>()
    //         .into_iter()
    //         .collect();
    //     for method_name in to_invalidate {
    //         graph.permanently_invalidate(method_name);
    //     };
    //     return Ok(graph);
    // }
}

#[pymethods]
impl DependencyCacheBase {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        return Self {
            cache: HashMap::new(),
            method_dependency_graph: MethodDependencyGraph::new(),
            call_stack: Vec::new(),
        };
    }

    // #[pyo3(signature = (*_args, **_kwargs))]
    // fn __init__(
    //     slf: &Bound<'_, Self>,
    //     _args: &Bound<'_, PyTuple>,
    //     _kwargs: Option<&Bound<'_, PyDict>>,
    // ) -> PyResult<()> {
    //     let cls = slf.get_type();
    //     let graph = Self::build_dependency_graph(&cls)?;
    //     slf.borrow_mut().method_dependency_graph = graph;
    //     return Ok(());
    // }

    pub fn get_cached_value(&self, py: Python<'_>, name: &str) -> Option<Py<PyAny>> {
        if self.method_dependency_graph.is_valid(name.to_string()) {
            return self.cache.get(name).map(|obj| obj.clone_ref(py));
        }
        return None;
    }

    pub fn update_cached_value(&mut self, name: &str, value: Py<PyAny>) {
        self.method_dependency_graph.temporarily_invalidate(name.to_string());
        self.cache.insert(name.to_string(), value);
        self.method_dependency_graph.validate(name.to_string());
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cached_values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.cache {
            dict.set_item(name, value.clone_ref(py))?;
        }
        return Ok(dict);
    }

    pub fn get_dependency_graph<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.method_dependency_graph.clone_graph() {
            dict.set_item(name, value)?;
        }
        return Ok(dict);
    }

    pub fn get_validation_state<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.method_dependency_graph.clone_state() {
            dict.set_item(name, value)?;
        }
        return Ok(dict);
    }
}
