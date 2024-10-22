use poke_engine::{
    abilities::Abilities,
    items::Items,
    state::{Pokemon, PokemonMoves, PokemonStatus, PokemonType},
};
use pyo3::{exceptions::PyValueError, prelude::*};
use std::str::FromStr;

use crate::pymove::PyMove;

#[derive(Clone)]
#[pyclass(name = "Pokemon")]
pub struct PyPokemon {
    pub pokemon: Pokemon,
}

impl PyPokemon {
    pub fn create_pokemon(&self) -> Pokemon {
        self.pokemon.clone()
    }

    pub fn create_fainted() -> Self {
        Self {
            pokemon: Pokemon {
                hp: 0,
                ..Default::default()
            },
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[pymethods]
impl PyPokemon {
    #[new]
    #[pyo3(signature = (
        id,
        pokedex_num=0,
        level=100,
        types=vec!["typeless".to_string(), "typeless".to_string()],
        hp=100,
        maxhp=100,
        ability=None,
        item=None,
        attack=100,
        defense=100,
        special_attack=100,
        special_defense=100,
        speed=100,
        status=None,
        rest_turns=0,
        sleep_turns=0,
        weight_kg=0.0,
        moves=vec![],
    ))]
    fn new(
        id: String,
        pokedex_num: i16,
        level: i8,
        mut types: Vec<String>,
        hp: i16,
        maxhp: i16,
        ability: Option<&str>,
        item: Option<&str>,
        attack: i16,
        defense: i16,
        special_attack: i16,
        special_defense: i16,
        speed: i16,
        status: Option<&str>,
        rest_turns: i8,
        sleep_turns: i8,
        weight_kg: f32,
        mut moves: Vec<PyMove>,
    ) -> PyResult<Self> {
        types.extend(std::iter::repeat("typeless".to_string()).take(2 - types.len()));
        moves.extend(std::iter::repeat(PyMove::create_empty_move()).take(6 - moves.len()));

        Ok(Self {
            pokemon: Pokemon {
                id,
                pokedex_num,
                level,
                types: (
                    PokemonType::deserialize(&types[0]),
                    PokemonType::deserialize(&types[1]),
                ),
                hp,
                maxhp,
                ability: match Abilities::from_str(ability.unwrap_or("none")) {
                    Ok(a) => a,
                    Err(()) => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid ability: {ability:?}"
                        )))
                    }
                },
                item: match Items::from_str(item.unwrap_or("NONE")) {
                    Ok(a) => a,
                    Err(()) => {
                        return Err(PyValueError::new_err(format!("Invalid item: {item:?}")))
                    }
                },
                attack,
                defense,
                special_attack,
                special_defense,
                speed,
                status: PokemonStatus::deserialize(status.unwrap_or("none")),
                rest_turns,
                sleep_turns,
                weight_kg,
                moves: PokemonMoves {
                    m0: moves[0].create_move(),
                    m1: moves[1].create_move(),
                    m2: moves[2].create_move(),
                    m3: moves[3].create_move(),
                    m4: moves[4].create_move(),
                    m5: moves[5].create_move(),
                },
            },
        })
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self.pokemon)
    }
}
