use poke_engine::{
    instruction::StateInstructions,
    state::{Side, State, StateTerrain, StateTrickRoom, StateWeather, Terrain, Weather},
};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::pyside::PySide;

#[pyclass(name = "State")]
pub struct PyState {
    pub state: State,
    prev_instructions: Option<Vec<StateInstructions>>,
}

#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
#[pymethods]
impl PyState {
    /// # Errors
    /// - Invalid weather type
    /// - Invalid terrain type
    #[new]
    #[pyo3(signature = (
        side_one=PySide::default(),
        side_two=PySide::default(),
        weather=None,
        weather_turns_remaining=None,
        terrain=None,
        terrain_turns_remaining=None,
        trick_room=None,
        trick_room_turns_remaining=None,
        team_preview=None
    ))]
    pub fn new(
        side_one: PySide,
        side_two: PySide,
        weather: Option<&str>,
        weather_turns_remaining: Option<i8>,
        terrain: Option<&str>,
        terrain_turns_remaining: Option<i8>,
        trick_room: Option<bool>,
        trick_room_turns_remaining: Option<i8>,
        team_preview: Option<bool>,
    ) -> PyResult<Self> {
        let weather = weather.unwrap_or("none");
        let terrain = terrain.unwrap_or("none");

        Ok(Self {
            state: State {
                side_one: side_one.side,
                side_two: side_two.side,

                // TODO: better error handling
                weather: StateWeather {
                    weather_type: match Weather::from_str(weather) {
                        Ok(w) => w,
                        Err(()) => {
                            return Err(PyValueError::new_err(format!(
                                "Invalid weather type: {weather}",
                            )))
                        }
                    },
                    turns_remaining: weather_turns_remaining.unwrap_or(-1),
                },
                terrain: StateTerrain {
                    terrain_type: match Terrain::from_str(terrain) {
                        Ok(w) => w,
                        Err(()) => {
                            return Err(PyValueError::new_err(format!(
                                "Invalid terrain type: {terrain}"
                            )))
                        }
                    },
                    turns_remaining: terrain_turns_remaining.unwrap_or(0),
                },
                trick_room: StateTrickRoom {
                    active: trick_room.unwrap_or(false),
                    turns_remaining: trick_room_turns_remaining.unwrap_or(0),
                },
                team_preview: team_preview.unwrap_or(false),
            },
            prev_instructions: None,
        })
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self.state)
    }
}
