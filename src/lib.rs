use pymove::PyMove;
use pyo3::prelude::*;
use pypokemon::PyPokemon;
use pyside::{PySide, PySideConditions};
use pystate::PyState;

mod pymove;
mod pypokemon;
mod pyside;
mod pystate;

/// A Python module implemented in Rust.
#[pymodule]
fn pokey_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    m.add_class::<PyState>()?;
    m.add_class::<PySide>()?;
    m.add_class::<PyMove>()?;
    m.add_class::<PyPokemon>()?;
    m.add_class::<PySideConditions>()?;
    Ok(())
}
