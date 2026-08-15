use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyString, PySet};

use std::collections::{HashMap};

use crate::dependency_graph::{MethodDependencyGraph, ValidationState};
use crate::normalise_method_args::normalise_and_hash_method;


#[pyclass(subclass)]
pub struct DependencyCacheBase {
    pub cache: HashMap<isize, Py<PyAny>>,
    pub method_dependency_graph: MethodDependencyGraph<isize, Py<PyTuple>>,
    pub call_stack: Vec<(isize, bool)>,
}

impl DependencyCacheBase {

    pub fn set_cached_value_by_hash(&mut self, hash: isize, value: Py<PyAny>) {
        if !self.method_dependency_graph.is_valid(hash) {
            return ();
        }
        self.cache.insert(hash, value);
    }

    pub fn validate_current_method(&mut self, hash: isize, use_cache: bool) {
        if !use_cache {
            self.method_dependency_graph.permanently_invalidate(hash);
        }

        let child_validation_states: Vec<isize> = self.method_dependency_graph.list_child_methods(&hash);

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
                    self.method_dependency_graph.permanently_invalidate(hash);
                }
                ValidationState::Invalid => {
                    self.method_dependency_graph.temporarily_invalidate(hash);
                }
                ValidationState::Valid => {
                    self.method_dependency_graph.validate(hash);
                }
            }
        }

    pub fn current_call_stack_top(&self) -> Option<(isize, bool)> {
        self.call_stack.last().cloned()
    }

    pub fn push_call_stack(&mut self, hash: isize, add_parent_dependencies: bool) {
        self.call_stack.push((hash, add_parent_dependencies));
    }

    pub fn pop_call_stack(&mut self) {
        self.call_stack.pop();
    }

    pub fn add_parent_dependency(&mut self, hash: isize, dependency: isize) {
        self.method_dependency_graph
            .add_parent_dependency(hash, vec![dependency]);
    }

    pub fn add_children_dependencies(&mut self, hash: isize, dependency: Vec<isize>, metadata: Py<PyTuple>) {
        self.method_dependency_graph
            .add_children_dependency(hash, dependency, metadata);
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
    pub fn get_cached_value_by_hash(&self, py: Python<'_>, hash: isize) -> Option<Py<PyAny>> {
        if self.method_dependency_graph.is_valid(hash) {
            return self.cache.get(&hash).map(|obj| obj.clone_ref(py));
        }
        return None;
    }

    #[pyo3(signature = (method_name, **kwargs))]
    pub fn get_cached_value(&self, py: Python<'_>, method_name: String, kwargs: Option<Py<PyDict>>) -> Option<Py<PyAny>> {
        let empty_args = PyTuple::empty(py);
        let kwargs_bound = kwargs.as_ref().map(|k| k.bind(py));
        let (hash, _) = normalise_and_hash_method(py, &method_name, None, &empty_args, kwargs_bound).ok()?;

        if self.method_dependency_graph.is_valid(hash) {
            self.cache.get(&hash).map(|obj| obj.clone_ref(py))
        } else {
            None
        }
    }

    #[pyo3(signature = (method_name, value, **kwargs))]
    pub fn update_cached_value(&mut self, py: Python<'_>, method_name: String, value: Py<PyAny>, kwargs: Option<Py<PyDict>>) {
        let empty_args = PyTuple::empty(py);
        let kwargs_bound = kwargs.as_ref().map(|k| k.bind(py));
        let Ok((hash, _)) = normalise_and_hash_method(py, &method_name, None, &empty_args, kwargs_bound) else {
            return;
        };

        self.method_dependency_graph.temporarily_invalidate(hash);
        self.cache.insert(hash, value);
        self.method_dependency_graph.validate(hash);
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

        for (child_hash, parent_hashes) in &self.method_dependency_graph.clone_graph() {
            let Some(child_meta) = self.method_dependency_graph.get_metadata(child_hash) else {
                continue;
            };
            let child_bound = child_meta.bind(py);

            let func_name: String = child_bound.get_item(0)?.extract()?;

            let args_item = child_bound.get_item(1)?;
            let normalized_args = args_item.cast::<PyTuple>()?;

            let child_key = if normalized_args.len() == 0 {
                PyString::new(py, &func_name).into_any()
            } else {
                child_bound.clone().into_any()
            };

            let mut parent_bounds = Vec::new();
            for parent_hash in parent_hashes {
                if let Some(parent_meta) = self.method_dependency_graph.get_metadata(parent_hash) {
                    let bound = parent_meta.bind(py);

                    let parent_name: String = bound.get_item(0)?.extract()?;
                    let parent_args_item = bound.get_item(1)?;
                    let parent_args = parent_args_item.cast::<PyTuple>()?;

                    let parent_key = if parent_args.len() == 0 {
                        PyString::new(py, &parent_name).into_any()
                    } else {
                        bound.clone().into_any()
                    };

                    parent_bounds.push(parent_key);
                }
            }

            let list = PySet::new(py, parent_bounds)?;
            dict.set_item(child_key, list)?;
        }
        Ok(dict)
    }

    pub fn get_validation_state<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (hash, state) in &self.method_dependency_graph.clone_state() {
            let Some(metadata) = self.method_dependency_graph.get_metadata(hash) else {
                continue;
            };
            let meta_bound = metadata.bind(py);

            let func_name: String = meta_bound.get_item(0)?.extract()?;

            let args_item = meta_bound.get_item(1)?;
            let normalized_args = args_item.cast::<PyTuple>()?;

            let key = if normalized_args.len() == 0 {
                PyString::new(py, &func_name).into_any()
            } else {
                meta_bound.clone().into_any()
            };

            dict.set_item(key, state)?;
        }
        Ok(dict)
    }
}
