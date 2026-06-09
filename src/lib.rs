use numpy::ndarray::Array2;
use numpy::{IntoPyArray, PyArray2};
use pyo3::prelude::*;

mod core;
mod ephys;

#[pyfunction]
fn zeros<'py>(py: Python<'py>, channels: usize, samples: usize) -> Bound<'py, PyArray2<i16>> {
    Array2::<i16>::zeros((channels, samples)).into_pyarray(py)
}

#[pymodule]
fn segovia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(zeros, m)?)?;
    Ok(())
}
