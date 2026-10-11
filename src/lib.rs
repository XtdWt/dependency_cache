mod decorator;
mod dependency_graph;
mod dependency_cache_base;
mod decorator_factory;
mod py_introspection_utils;
mod metadata_hash_manager;
mod scenario;

use pyo3::prelude::*;

use dependency_cache_base::DependencyCacheBase;
use decorator_factory::{ManualDependencyCacheDecoratorFactory, AutomagicDependencyCacheDecoratorFactory};
use scenario::custom_scenario;


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
}
