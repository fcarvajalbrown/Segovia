use pyo3::prelude::*;

mod core;
mod ephys;

#[pymodule]
fn segovia(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
