use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyString, PySet, PyType};
use pyo3::exceptions::{PyValueError, PyKeyError};

use rand::rng;
use rand::seq::SliceRandom;

use std::collections::{HashMap, HashSet};

use crate::dependency_graph::{MethodDependencyGraph, ValidationState};
use crate::py_introspection_utils::{normalise_function_signature_and_hash, validate_self_only_method};
use crate::decorator::DependencyCacheDecorator;


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

    pub fn get_cached_value_by_hash(&self, py: Python<'_>, hash: isize) -> Option<Py<PyAny>> {
        if self.method_dependency_graph.is_valid(hash) {
            return self.cache.get(&hash).map(|obj| obj.clone_ref(py));
        }
        return None;
    }

    fn create_hash(&self, py: Python<'_>, method_name: &String, kwargs: &Option<Py<PyDict>>) -> Option<isize> {
        let empty_args = PyTuple::empty(py);
        let kwargs_bound = kwargs.as_ref().map(|k| k.bind(py));
        let (hash, _) = normalise_function_signature_and_hash(py, &method_name, None, &empty_args, kwargs_bound).ok()?;
        return Some(hash);
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
            call_stack: Vec::new(),
        };
    }

    #[pyo3(signature = (method_name, **kwargs))]
    pub fn get_cached_value(&self, py: Python<'_>, method_name: String, kwargs: Option<Py<PyDict>>) -> Option<Py<PyAny>> {
        let hash = self.create_hash(py, &method_name, &kwargs)?;

        if self.method_dependency_graph.is_valid(hash) {
            self.cache.get(&hash).map(|obj| obj.clone_ref(py))
        } else {
            None
        }
    }

    #[pyo3(signature = (method_name, value, **kwargs))]
    pub fn update_cached_value(&mut self, py: Python<'_>, method_name: String, value: Py<PyAny>, kwargs: Option<Py<PyDict>>) -> PyResult<()> {
        let maybe_hash = self.create_hash(py, &method_name, &kwargs);
        let Some(hash) = maybe_hash else {
            return Ok(());
        };

        let Some(v) = self.cache.get_mut(&hash) else {
            return Err(PyKeyError::new_err(format!(
                "Cannot update cached value for '{method_name}'"
            )));
        };
        *v = value;
        self.method_dependency_graph.temporarily_invalidate(hash);
        self.method_dependency_graph.validate(hash);
        return Ok(());
    }

    pub fn clear_cache(&mut self) {
        self.method_dependency_graph.invalidate_all();
        self.cache.clear();
    }

    #[pyo3(signature = (method_name, **kwargs))]
    pub fn clear_cached_value(&mut self, py: Python<'_>, method_name: String, kwargs: Option<Py<PyDict>>) -> PyResult<()> {
        let maybe_hash = self.create_hash(py, &method_name, &kwargs);
        let Some(hash) = maybe_hash else {
            return Ok(());
        };

        let Some(_) = self.cache.get_mut(&hash) else {
            return Err(PyKeyError::new_err(format!(
                "Cannot clear cached value for '{method_name}'"
            )));
        };

        self.method_dependency_graph.temporarily_invalidate(hash);
        self.cache.remove(&hash);
        return Ok(());
    }

    pub fn get_cached_values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (hash, value) in &self.cache {
            let Some(meta) = self.method_dependency_graph.get_metadata(hash) else {
                continue;
            };
            let bound = meta.bind(py);

            let func_name: String = bound.get_item(0)?.extract()?;

            let args_item = bound.get_item(1)?;
            let normalized_args = args_item.cast::<PyTuple>()?;

            let key = if normalized_args.len() == 0 {
                PyString::new(py, &func_name).into_any()
            } else {
                bound.clone().into_any()
            };

            dict.set_item(key, value.clone_ref(py))?;
        }
        Ok(dict)
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

    #[pyo3(signature = (order=None))]
    pub fn dump_cache<'py>(slf: &Bound<'py, Self>, py: Python<'py>, order: Option<Vec<String>>) -> PyResult<Bound<'py, PyDict>> {
        let cls = slf.get_type();

        let mut visited = HashSet::new();
        let mut serialisable_methods = Vec::new();

        for mro_class in cls.mro().iter() {
            let mro_class: Bound<'_, PyType> = mro_class.extract()?;
            let namespace = mro_class.getattr("__dict__")?;

            let method_names = namespace.call_method0("items")?.try_iter()?;
            for method_name in method_names {
                let (name, value): (String, Bound<'_, PyAny>) = method_name?.extract()?;
                if name.starts_with("__") || !visited.insert(name.clone()) {
                    continue;
                }
                if let Ok(decorator) = value.cast::<DependencyCacheDecorator>() {
                    if decorator.borrow().serialisable {
                        serialisable_methods.push(name);
                    }
                }
            }
        }

        // methods from order first
        let mut ordered_methods = Vec::new();
        let mut remaining: HashSet<String> = serialisable_methods.into_iter().collect();
        if let Some(order_vec) = order {
            let mut seen = HashSet::new();
            for name in order_vec {
                if seen.insert(name.clone()) && remaining.remove(&name) {
                    ordered_methods.push(name);
                }
            }
        }
        // randomise for the rest of the methods
        let mut remaining_vec: Vec<String> = remaining.into_iter().collect();
        remaining_vec.shuffle(&mut rng());
        ordered_methods.extend(remaining_vec);

        let result = PyDict::new(py);
        for method_name in ordered_methods {
            let value = slf.call_method(&method_name, (), None)?;
            result.set_item(method_name, value)?;
        }

        Ok(result)
    }

    #[pyo3(signature = (data, order=None))]
    pub fn load_cache<'py>(
        slf: &Bound<'_, Self>,
        py: Python<'py>,
        data: Bound<'py, PyDict>,
        order: Option<Vec<String>>,
    ) -> PyResult<()> {
        let all_keys: Vec<String> = data
            .keys()
            .into_iter()
            .map(|k| k.extract::<String>())
            .collect::<PyResult<Vec<_>>>()?;

        let class = slf.get_type();

        let load_method = |name: &str| -> PyResult<()> {
            let loaded_value = match data.get_item(name)? {
                Some(val) => val,
                None => return Ok(()),
            };

            let method_attr = class.getattr(name)?;

            let decorator = method_attr
                .cast::<DependencyCacheDecorator>()
                .map_err(|_| {
                    PyValueError::new_err(format!(
                        "Method '{}' is not decorated with @dependency_cached",
                        name
                    ))
                })?;

            let decorator_ref = decorator.borrow();


            if !decorator_ref.serialisable {
                return Err(PyValueError::new_err(format!(
                    "Method '{}' is not marked as serialisable",
                    name
                )));
            }

            let underlying_func = decorator_ref.func.bind(py);
            validate_self_only_method(py, &underlying_func)?;

            let _ = slf.call_method(name, (), None)?;

            let empty_args = PyTuple::empty(py);
            let (hash, _) = normalise_function_signature_and_hash(py, name, None, &empty_args, None)
                .map_err(|e| PyValueError::new_err(format!("Failed to hash '{}': {}", name, e)))?;
            slf.borrow_mut().cache.insert(hash, loaded_value.unbind());
            Ok(())
        };

        let mut loaded = HashSet::new();

        // methods from order first
        if let Some(order_vec) = order {
            for name in order_vec {
                if loaded.contains(&name) {
                    continue;
                }
                load_method(&name)?;
                loaded.insert(name.to_string());
            }
        }

        let mut remaining_keys: Vec<String> = all_keys
            .into_iter()
            .filter(|name| !loaded.contains(name))
            .collect();

        // randomise for the rest of the methods
        remaining_keys.shuffle(&mut rng());
        for name in remaining_keys {
            load_method(&name)?;
        }

        Ok(())
    }
}
