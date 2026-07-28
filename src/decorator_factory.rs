use pyo3::prelude::*;

use crate::decorator::DependencyCacheDecorator;


#[pyclass(name = "dependency_cached", frozen)]
pub struct DependencyCacheDecoratorFactory {
    use_cache: bool,
    dependencies: Vec<String>,
}

#[pymethods]
impl DependencyCacheDecoratorFactory {
    #[new]
    #[pyo3(signature = (use_cache=true, dependencies=vec![]))]
    fn new(use_cache: bool, dependencies: Vec<String>) -> Self {
        return Self {
            use_cache,
            dependencies,
        };
    }

    fn __call__(&self, py: Python<'_>, func: Py<PyAny>) -> PyResult<DependencyCacheDecorator> {
        let method_name: String = func.getattr(py, "__name__")?.extract(py)?;
        return Ok(
            DependencyCacheDecorator {
                func,
                use_cache: self.use_cache,
                dependencies: self.dependencies.clone(),
                method_name,
            }
        );
    }
}
