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
