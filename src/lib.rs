use pymove::PyMove;
use pyo3::prelude::*;
use pypokemon::PyPokemon;
use pystate::PyState;

mod pymove;
mod pypokemon;
mod pystate;

/// A Python module implemented in Rust.
#[pymodule]
fn pokey_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    m.add_class::<PyState>()?;
    m.add_class::<PyMove>()?;
    m.add_class::<PyPokemon>()?;
    Ok(())
}
