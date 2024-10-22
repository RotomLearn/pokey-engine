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
    #[pyo3(signature = (id, move_num, pp, disabled=None))]
    fn new(id: &str, move_num: i16, pp: i8, disabled: Option<bool>) -> PyResult<Self> {
        let Ok(choice) = Choices::from_str(id) else {
            return Err(PyValueError::new_err(format!("Invalid move id: {id}",)));
        };

        Ok(Self {
            r#move: Move {
                id: choice,
                move_num,
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

#[derive(Debug)]
#[pyclass(name = "MoveChoice")]
pub enum PyMoveChoice {
    Move(String),
    Switch(u8),
    None(),
}

#[pymethods]
impl PyMoveChoice {
    fn __repr__(&self) -> String {
        match self {
            Self::Move(m) => format!("Move {m}"),
            Self::Switch(s) => format!("Switch {s}"),
            Self::None() => "None".to_string(),
        }
    }

    fn __str__(&self) -> String {
        match self {
            Self::Move(m) => m.to_string(),
            Self::Switch(s) => format!("switch {s}"),
            Self::None() => "None".to_string(),
        }
    }
}
