mod decorator;
mod dependency_graph;
mod dependency_cache_base;
mod decorator_factory;

use pyo3::prelude::*;

use dependency_cache_base::DependencyCacheBase;
use decorator_factory::{ManualDependencyCacheDecoratorFactory, AutomagicDependencyCacheDecoratorFactory};


#[pymodule]
mod dependency_cache {

    #[pymodule_export]
    use super::DependencyCacheBase;

    #[pymodule_export]
    use super::ManualDependencyCacheDecoratorFactory;

    #[pymodule_export]
    use super::AutomagicDependencyCacheDecoratorFactory;
}
