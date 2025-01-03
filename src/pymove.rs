use poke_engine::{
    choices::{Choices, MOVES},
    state::Move,
};
use pyo3::{exceptions::PyValueError, prelude::*};
use std::str::FromStr;

#[derive(Clone)]
#[pyclass(name = "Move")]
pub struct PyMove {
    pub r#move: Move,
}

impl PyMove {
    pub fn create_move(&self) -> Move {
        self.r#move.clone()
    }
    pub fn create_empty_move() -> Self {
        Self {
            r#move: Move {
                disabled: true,
                pp: 0,
                ..Default::default()
            },
        }
    }
}

#[pymethods]
impl PyMove {
    #[new]
    #[pyo3(signature = (id, pp, disabled=None))]
    fn new(id: &str, pp: i8, disabled: Option<bool>) -> PyResult<Self> {
        let Ok(choice) = Choices::from_str(id) else {
            return Err(PyValueError::new_err(format!("Invalid move id: {id}",)));
        };

        Ok(Self {
            r#move: Move {
                id: choice,
                disabled: disabled.unwrap_or(false),
                pp,
                choice: match MOVES.get(&choice) {
                    Some(m) => m.clone(),
                    None => {
                        return Err(PyValueError::new_err(format!("Invalid choice: {choice}",)))
                    }
                },
            },
        })
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self.r#move)
    }
}

// TODO: representing switch as the pokemon species ID is not unique
// multiple pokemon of the same species on the same team would cause issues
#[derive(Debug)]
#[pyclass(name = "MoveChoice")]
pub enum PyMoveChoice {
    Move(String),
    MoveTera(String),
    Switch(String),
    None(),
}

#[pymethods]
impl PyMoveChoice {
    fn __repr__(&self) -> String {
        match self {
            Self::Move(m) => format!("Move {m}"),
            Self::MoveTera(m) => format!("Move Tera {m}"),
            Self::Switch(s) => format!("Switch {s}"),
            Self::None() => "None".to_string(),
        }
    }

    fn __str__(&self) -> String {
        match self {
            Self::Move(x) | Self::MoveTera(x) | Self::Switch(x) => x.to_string(),
            Self::None() => "None".to_string(),
        }
    }
}
