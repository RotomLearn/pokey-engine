use pokezoo::observations;
use pymove::PyMove;
use pyo3::prelude::*;
use pypokemon::PyPokemon;
use pyside::{PySide, PySideConditions};
use pystate::PyState;

mod mcts_ol;
mod mcts_ol_st;
mod pokezoo;
mod pymove;
mod pypokemon;
mod pyside;
mod pystate;

#[allow(clippy::wildcard_imports)]
#[pymodule]
mod pokey_engine {
    use super::*;

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<PyState>()?;
        m.add_class::<PySide>()?;
        m.add_class::<PyMove>()?;
        m.add_class::<PyPokemon>()?;
        m.add_class::<PySideConditions>()?;

        Ok(())
    }

    #[pymodule(name = "pokezoo")]
    mod pypokezoo {
        use super::*;

        #[pymodule_init]
        fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
            m.add_function(wrap_pyfunction!(observations, m)?)?;

            Python::with_gil(|py| {
                py.import_bound("sys")?
                    .getattr("modules")?
                    .set_item("pokey_engine.pokezoo", m)
            })
        }
    }
}
