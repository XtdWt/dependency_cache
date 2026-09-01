use std::collections::HashSet;

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyDict, PyString, PyTuple, PyList};


pub fn normalise_function_signature_and_hash<'py>(
    py: Python<'py>,
    func_name: &str,
    func: Option<&Bound<'py, PyAny>>,
    args: &Bound<'py, PyTuple>,
    kwargs: Option<&Bound<'py, PyDict>>,
) -> PyResult<(isize, Bound<'py, PyTuple>)> {
    let dict = match func {
        Some(f) => {
            let inspect = py.import("inspect")?;
            let bound = inspect
                .getattr("signature")?
                .call1((f,))?
                .getattr("bind")?
                .call(args, kwargs)?;
            bound.getattr("apply_defaults")?.call0()?;
            bound.getattr("arguments")?
        }
        None => match kwargs {
            Some(d) => d.clone().into_any(),
            None => PyDict::new(py).into_any(),
        },
    };

    let dict = dict.cast::<PyDict>()?;

    let mut items: Vec<(String, Bound<'py, PyAny>)> = Vec::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            if key == "self" {
                continue;
            }
            items.push((key, v));
        }
    items.sort_by(|a, b| a.0.cmp(&b.0));


    let pairs = items
        .into_iter()
        .map(|(k, v)| {
            let k_py = PyString::new(py, &k).into_any();
            PyTuple::new(py, vec![k_py, v])
        })
        .collect::<PyResult<Vec<_>>>()?;

    let normalised = PyTuple::new(py, pairs)?;

    let combined = PyTuple::new(
        py,
        vec![
            PyString::new(py, func_name).into_any(),
            normalised.clone().into_any(),
        ],
    )?;
    let hash_val = combined.hash()?;

    return Ok((hash_val, combined));
}


pub fn validate_self_only_method(py: Python<'_>, func: &Bound<'_, PyAny>) -> PyResult<()> {
    let inspect = py.import("inspect")?;
    let builtins = py.import("builtins")?;
    let signature = inspect.call_method1("signature", (func,))?;
    // really ugly, but the inspect.Signature type is not extractable
    let parameters = signature.getattr("parameters")?;
    let list_obj = builtins.call_method1("list", (parameters,))?;
    let param_names: Vec<String> = list_obj.extract()?;

    if param_names.len() != 1 || param_names[0] != "self" {
        let msg = format!(
            "Serialisable method must have exactly one parameter named 'self', got: {:?}",
            param_names
        );
        return Err(PyValueError::new_err(msg));
    }
    Ok(())
}


pub fn parse_dependencies(py: Python<'_>, dep_list: &Bound<'_, PyAny>) -> PyResult<Vec<(String, Py<PyDict>)>> {
    let list = dep_list.cast::<PyList>()?;
    let mut parsed = Vec::with_capacity(list.len());

    for item in list.iter() {
        if let Ok(tuple) = item.cast::<PyTuple>() {
            if tuple.len() == 2 {
                let method_name: String = tuple.get_item(0)?.extract()?;
                let kwargs = tuple.get_item(1)?;
                let kwargs_dict = kwargs.cast::<PyDict>()?;
                parsed.push((method_name, kwargs_dict.clone().unbind()));
                continue;
            }
        }

        if let Ok(method_name) = item.extract::<String>() {
            let empty_dict = PyDict::new(py);
            parsed.push((method_name, empty_dict.unbind()));
            continue;
        }

        return Err(PyValueError::new_err(
            "Dependency must be a tuple (str, dict) or a str",
        ));
    }

    return Ok(parsed);
}


fn visit(
    py: Python<'_>,
    node: &Bound<'_, PyAny>,
    ast_module: &Bound<'_, PyModule>,
    dependencies: &mut Vec<(String, Py<PyDict>)>,
    visited: &mut HashSet<String>,
) -> PyResult<()> {
    let ast_call = ast_module.getattr("Call")?;
    let ast_attribute = ast_module.getattr("Attribute")?;
    let ast_name = ast_module.getattr("Name")?;

    let attr = (|| -> PyResult<Option<String>> {
        if !node.is_instance(&ast_call)? {
            return Ok(None);
        }
        let args = node.getattr("args")?;
        let keywords = node.getattr("keywords")?;
        if args.len()? != 0 || keywords.len()? != 0 {
            return Ok(None);
        }
        let func = node.getattr("func")?;
        if !func.is_instance(&ast_attribute)? {
            return Ok(None);
        }
        let value = func.getattr("value")?;
        if !value.is_instance(&ast_name)? {
            return Ok(None);
        }
        let id: String = value.getattr("id")?.extract()?;
        if id != "self" {
            return Ok(None);
        }
        Ok(Some(func.getattr("attr")?.extract()?))
    })()?;

    if let Some(attr) = attr {
        if visited.insert(attr.clone()) {
            let empty_dict = PyDict::new(py);
            dependencies.push((attr, empty_dict.unbind()));
        }
    }


    for child in ast_module
        .call_method1("iter_child_nodes", (node,))?
        .try_iter()?
    {
        visit(py, &child?, ast_module, dependencies, visited)?;
    }

    Ok(())
}


pub fn ast_inspect_method_dependencies(py: Python<'_>, python_func: &Py<PyAny>) -> PyResult<Vec<(String, Py<PyDict>)>> {
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

    let mut dependencies: Vec<(String, Py<PyDict>)> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    for node in ast_tree.getattr("body")?.try_iter()? {
        let node = node?;
        if !node.is_instance(&ast_function)? && !node.is_instance(&ast_async_function)? {
            continue;
        }
        for statement in node.getattr("body")?.try_iter()? {
            let statement = statement?;
            visit(py, &statement, &ast_module, &mut dependencies, &mut visited)?
        }
    };
    return Ok(dependencies);
}
