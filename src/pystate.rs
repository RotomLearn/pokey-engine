use poke_engine::{
    evaluate::evaluate,
    generate_instructions::generate_instructions_from_move_pair,
    instruction::StateInstructions,
    state::{
        MoveChoice, Side, State, StateTerrain, StateTrickRoom, StateWeather, Terrain, Weather,
    },
};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::{pymove::PyMoveChoice, pyside::PySide};

#[pyclass(name = "State")]
pub struct PyState {
    pub state: State,

    // instructions from generate_instructions()
    prev_instructions: Option<Vec<StateInstructions>>,

    // stack of instructions from apply_instructions()
    instruction_stack: Vec<StateInstructions>,
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
            instruction_stack: vec![],
        })
    }

    fn battle_is_over(&self) -> f32 {
        self.state.battle_is_over()
    }

    fn get_all_options(&self) -> (Vec<PyMoveChoice>, Vec<PyMoveChoice>) {
        fn convert_move_choice(choice: MoveChoice, side: &Side) -> PyMoveChoice {
            match choice {
                MoveChoice::Move(m) => PyMoveChoice::Move(
                    side.get_active_immutable().moves[m]
                        .id
                        .to_string()
                        .to_lowercase(),
                ),
                MoveChoice::Switch(s) => PyMoveChoice::Switch(side.pokemon[s].id.clone()),
                MoveChoice::None => PyMoveChoice::None(),
            }
        }

        let (side1, side2) = self.state.get_all_options();

        (
            side1
                .into_iter()
                .map(|c| convert_move_choice(c, &self.state.side_one))
                .collect::<Vec<PyMoveChoice>>(),
            side2
                .into_iter()
                .map(|c| convert_move_choice(c, &self.state.side_two))
                .collect::<Vec<PyMoveChoice>>(),
        )
    }

    fn generate_instructions(
        &mut self,
        side_one_move: String,
        side_two_move: String,
    ) -> PyResult<Vec<PyStateInstructions>> {
        let Some(s1_move) = self.state.side_one.string_to_movechoice(&side_one_move) else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid move for s1: {side_one_move}"
            )));
        };

        let Some(s2_move) = self.state.side_two.string_to_movechoice(&side_two_move) else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid move for s2: {side_two_move}"
            )));
        };

        let instructions =
            generate_instructions_from_move_pair(&mut self.state, &s1_move, &s2_move);

        let py_instructions = instructions
            .iter()
            .map(PyStateInstructions::from_state_instructions)
            .collect();

        self.prev_instructions = Some(instructions);

        Ok(py_instructions)
    }

    fn apply_instructions(&mut self, index: usize) -> PyResult<()> {
        let Some(ref generated_instructions) = self.prev_instructions else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Generate instructions must be called first".to_string(),
            ));
        };

        let Some(instructions) = generated_instructions.get(index) else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid index: {index}"
            )));
        };

        self.instruction_stack.push(instructions.clone());

        self.state
            .apply_instructions(&instructions.instruction_list);

        Ok(())
    }

    fn reverse_last_instructions(&mut self) -> PyResult<()> {
        let Some(instructions) = self.instruction_stack.pop() else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No instructions to reverse".to_string(),
            ));
        };

        self.state
            .reverse_instructions(&instructions.instruction_list);

        Ok(())
    }

    fn evaluate(&self) -> f32 {
        evaluate(&self.state)
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self.state)
    }
}

#[derive(Clone)]
#[pyclass(get_all, set_all)]
struct PyStateInstructions {
    pub percentage: f32,
    pub instruction_list: Vec<String>,
}

impl PyStateInstructions {
    fn from_state_instructions(instructions: &StateInstructions) -> Self {
        Self {
            percentage: instructions.percentage,
            instruction_list: instructions
                .instruction_list
                .iter()
                .map(|i| format!("{i:?}"))
                .collect(),
        }
    }
}
