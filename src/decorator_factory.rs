use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

use std::collections::HashSet;
use crate::decorator::DependencyCacheDecorator;


#[pyclass(name = "dependency_cached", frozen)]
pub struct ManualDependencyCacheDecoratorFactory {
    use_cache: bool,
    dependencies: Vec<String>,
}

#[pymethods]
impl ManualDependencyCacheDecoratorFactory {
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


#[pyclass(name = "automagically_dependency_cached", frozen)]
pub struct AutomagicDependencyCacheDecoratorFactory {
    use_cache: bool,
    dependencies: Vec<String>,
}

#[pymethods]
impl AutomagicDependencyCacheDecoratorFactory {
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
        let dependencies = if self.dependencies.is_empty() {
            parse_dependencies_from_func(py, &func)?
        } else {
            self.dependencies.clone()
        };
        return Ok(
            DependencyCacheDecorator {
                func,
                use_cache: self.use_cache,
                dependencies: dependencies,
                method_name,
            }
        );
    }
}

fn visit(node: &Bound<'_, PyAny>, ast_module: &Bound<'_, PyModule>, dependencies: &mut Vec<String>, visited: &mut HashSet<String>) -> PyResult<()> {
    let ast_call = ast_module.getattr("Call")?;
    let ast_attribute = ast_module.getattr("Attribute")?;
    let ast_name = ast_module.getattr("Name")?;

    if node.is_instance(&ast_call)? {
        let func = node.getattr("func")?;
        if func.is_instance(&ast_attribute)? {
            let value = func.getattr("value")?;
            if value.is_instance(&ast_name)? {
                let id: String = value.getattr("id")?.extract()?;
                if id == "self" {
                    let attr: String = func.getattr("attr")?.extract()?;
                    if visited.insert(attr.clone()) {
                        dependencies.push(attr);
                    }
                }
            }
        }
    }

    for child in ast_module
        .call_method1("iter_child_nodes", (node,))?
        .try_iter()?
    {
        visit(&child?, ast_module, dependencies, visited)?;
    }

    Ok(())
}


fn parse_dependencies_from_func(py: Python<'_>, python_func: &Py<PyAny>) -> PyResult<Vec<String>> {
    let inspect = PyModule::import(py, "inspect")?;
    let function_string: String = inspect
        .call_method1("getsource", (python_func.bind(py),))?
        .extract()?;
    let textwrap = PyModule::import(py, "textwrap")?;
    let function_string_clean: String = textwrap
        .call_method1("dedent", (function_string,))?
        .call_method0("strip")?
        .extract()?;
    let ast_module = PyModule::import(py, "ast")?;
    let ast_tree = ast_module
        .call_method1("parse", (function_string_clean,))
        .map_err(|e| PyValueError::new_err(format!("failed to parse source: {e}")))?;

    let ast_function = ast_module.getattr("FunctionDef")?;
    let ast_async_function = ast_module.getattr("AsyncFunctionDef")?;

    let mut dependencies: Vec<String> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    for node in ast_tree.getattr("body")?.try_iter()? {
        let node = node?;
        if node.is_instance(&ast_function)? || node.is_instance(&ast_async_function)? {
            for statement in node.getattr("body")?.try_iter()? {
                let statement = statement?;
                visit(&statement, &ast_module, &mut dependencies, &mut visited)?
            }
        }
    };
    return Ok(dependencies);
}
