use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple};


pub fn normalise_and_hash_method<'py>(
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
