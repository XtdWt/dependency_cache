mod decorator;
mod dependency_graph;
mod dependency_cache_base;

use pyo3::prelude::*;

use decorator::DependencyCacheDecorator;
use dependency_cache_base::DependencyCacheBase;

#[pyclass(name = "dependency_cached", frozen)]
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


#[pymodule]
mod dependency_cache {

    #[pymodule_export]
    use super::DependencyCacheBase;

    #[pymodule_export]
    use super::DependencyCacheDecoratorFactory;
}
