mod dependency_graph;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use std::collections::HashMap;

use dependency_graph::MethodDependencyGraph;

#[pyclass(subclass)]
pub struct DependencyCacheBase {
    cache: HashMap<String, Py<PyAny>>,
    method_dependency_graph: MethodDependencyGraph,
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

    fn get_cached_value(&self, py: Python<'_>, name: &str) -> Option<Py<PyAny>> {
        if self.method_dependency_graph.is_valid(name.to_string()) {
            return self.cache.get(name).map(|obj| obj.clone_ref(py));
        }
        None
    }

    fn set_cached_value(&mut self, name: &str, value: Py<PyAny>) {
        self.cache.insert(name.to_string(), value);
        self.method_dependency_graph.validate(name.to_string());
    }

    fn update_cached_value(&mut self, name: &str, value: Py<PyAny>) {
        self.method_dependency_graph.invalidate(name.to_string());
        self.cache.insert(name.to_string(), value);
        self.method_dependency_graph.validate(name.to_string());
    }

    fn clear_cache(&mut self) {
        self.cache.clear();
    }

    fn current_cache<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.cache {
            dict.set_item(name, value.clone_ref(py))?;
        }
        Ok(dict)
    }

    fn current_graph<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.method_dependency_graph.cache_dependency_graph {
            dict.set_item(name, value)?;
        }
        Ok(dict)
    }

    fn current_cache_validation<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (name, value) in &self.method_dependency_graph.cache_validation {
            dict.set_item(name, value)?;
        }
        Ok(dict)
    }
}


#[pyclass(name = "dependency_cache", frozen)]
pub struct DependencyCacheDecoratorFactory {
    use_cache: bool,
    dependencies: Vec<String>
}

#[pymethods]
impl DependencyCacheDecoratorFactory {
    #[new]
    #[pyo3(signature = (use_cache=true, dependencies=vec![]))]
    fn new(use_cache: bool, dependencies: Vec<String>) -> Self {
        Self { use_cache, dependencies }
    }

    fn __call__(&self, py: Python<'_>, func: Py<PyAny>) -> PyResult<DependencyCacheDecorator> {
        let method_name: String = func.getattr(py, "__name__")?.extract(py)?;
        Ok(DependencyCacheDecorator {
            func,
            use_cache: self.use_cache,
            dependencies: self.dependencies.clone(),
            method_name,
        })
    }
}

#[pyclass(frozen)]
pub struct DependencyCacheDecorator {
    func: Py<PyAny>,
    use_cache: bool,
    dependencies: Vec<String>,
    method_name: String,
}

#[pymethods]
impl DependencyCacheDecorator {

    fn __set__(
        &self,
        _obj: Py<PyAny>,
        _value: Py<PyAny>
    ) -> PyResult<()> {
        Err(PyTypeError::new_err("cannot assign to decorated method"))
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
        Ok(bound_method.unbind())
    }

    #[pyo3(signature = (*args, **kwargs))]
    fn __call__(
        &self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let func = self.func.bind(py);

        if !self.use_cache {
            return Ok(func.call(args, kwargs)?.unbind());
        }

        if args.is_empty() {
            return Err(PyValueError::new_err(
                "cache expected to be used as an instance method decorator",
            ));
        }

        let instance = args.get_item(0)?;
        let base = instance.cast::<DependencyCacheBase>().map_err(|_| {
            PyTypeError::new_err(
                "the decorated method's class must inherit from DependencyCacheBase",
            )
        })?;

        if let Some(cached) = base.borrow().get_cached_value(py, &self.method_name) {
            return Ok(cached);
        }
        let result = func.call(args, kwargs)?.unbind();
        base.borrow_mut()
            .method_dependency_graph
            .add_dependency(self.method_name.clone(), self.dependencies.clone());

        base.borrow_mut()
            .set_cached_value(&self.method_name, result.clone_ref(py));

        Ok(result)
    }
}

#[pymodule]
fn dependency_cache(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DependencyCacheBase>()?;
    m.add_class::<DependencyCacheDecoratorFactory>()?;
    m.add_class::<DependencyCacheDecorator>()?;
    Ok(())
}
