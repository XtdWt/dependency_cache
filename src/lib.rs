mod decorator;
mod dependency_graph;
mod dependency_cache_base;
mod decorator_factory;
mod py_introspection_utils;
mod metadata_hash_manager;
mod scenario;

use pyo3::prelude::*;
use rayon::prelude::*;

use dependency_cache_base::DependencyCacheBase;
use decorator_factory::{ManualDependencyCacheDecoratorFactory, AutomagicDependencyCacheDecoratorFactory};
use scenario::custom_scenario;



#[pyfunction]
fn parallel_call(py: Python<'_>, objects: Vec<Py<PyAny>>, method_name: String) -> PyResult<Vec<Py<PyAny>>> {
    let results: Vec<Py<PyAny>> = py.detach(|| {
        objects
            .par_iter()
            .map(|obj| {
                Python::attach(|py| -> PyResult<Py<PyAny>> {
                    let bound = obj.bind(py);
                    let result = bound.call_method0(method_name.clone())?;
                    let unbound_res = result.unbind();
                    Ok(unbound_res)
                })
            })
            .collect::<PyResult<Vec<Py<PyAny>>>>() // Collect the PyResult
    })?;

    Ok(results)
}


#[pymodule]
mod dependency_cache {

    #[pymodule_export]
    use super::DependencyCacheBase;

    #[pymodule_export]
    use super::ManualDependencyCacheDecoratorFactory;

    #[pymodule_export]
    use super::AutomagicDependencyCacheDecoratorFactory;

    #[pymodule_export]
    use super::custom_scenario;

    #[pymodule_export]
    use super::parallel_call;
}
