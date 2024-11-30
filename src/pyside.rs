use std::collections::HashSet;

use poke_engine::{
    choices::Choices,
    state::{
        DamageDealt, LastUsedMove, PokemonIndex, PokemonVolatileStatus, Side, SideConditions,
        SidePokemon,
    },
};
use pyo3::{exceptions::PyValueError, prelude::*};
use std::str::FromStr;

use crate::pypokemon::PyPokemon;

#[derive(Clone, Default)]
#[pyclass(name = "SideConditions")]
pub struct PySideConditions {
    pub side_conditions: SideConditions,
}

#[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
#[pymethods]
impl PySideConditions {
    #[new]
    #[pyo3(signature = (
        spikes=0,
        toxic_spikes=0,
        stealth_rock=0,
        sticky_web=0,
        tailwind=0,
        lucky_chant=0,
        lunar_dance=0,
        reflect=0,
        light_screen=0,
        aurora_veil=0,
        crafty_shield=0,
        safeguard=0,
        mist=0,
        protect=0,
        healing_wish=0,
        mat_block=0,
        quick_guard=0,
        toxic_count=0,
        wide_guard=0,
    ))]
    const fn new(
        spikes: i8,
        toxic_spikes: i8,
        stealth_rock: i8,
        sticky_web: i8,
        tailwind: i8,
        lucky_chant: i8,
        lunar_dance: i8,
        reflect: i8,
        light_screen: i8,
        aurora_veil: i8,
        crafty_shield: i8,
        safeguard: i8,
        mist: i8,
        protect: i8,
        healing_wish: i8,
        mat_block: i8,
        quick_guard: i8,
        toxic_count: i8,
        wide_guard: i8,
    ) -> Self {
        Self {
            side_conditions: SideConditions {
                spikes,
                toxic_spikes,
                stealth_rock,
                sticky_web,
                tailwind,
                lucky_chant,
                lunar_dance,
                reflect,
                light_screen,
                aurora_veil,
                crafty_shield,
                safeguard,
                mist,
                protect,
                healing_wish,
                mat_block,
                quick_guard,
                toxic_count,
                wide_guard,
            },
        }
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self.side_conditions)
    }
}

#[derive(Clone, Default)]
#[pyclass(name = "Side")]
pub struct PySide {
    pub side: Side,
}

#[allow(
    clippy::fn_params_excessive_bools,
    clippy::too_many_arguments,
    clippy::needless_pass_by_value
)]
#[pymethods]
impl PySide {
    #[new]
    #[pyo3(signature = (
        active_index=0,
        baton_passing=false,
        pokemon=vec![],
        side_conditions=PySideConditions::default(),
        wish=(0, 0),
        future_sight=(0, 0),
        force_switch=false,
        force_trapped=false,
        slow_uturn_move=false,
        volatile_statuses=vec![],
        substitute_health=0,
        attack_boost=0,
        defense_boost=0,
        special_attack_boost=0,
        special_defense_boost=0,
        speed_boost=0,
        accuracy_boost=0,
        evasion_boost=0,
        last_used_move="move:none",
        switch_out_move_second_saved_move="none",
    ))]
    fn new(
        active_index: u8,
        baton_passing: bool,
        mut pokemon: Vec<PyPokemon>,
        side_conditions: PySideConditions,
        wish: (i8, i16),
        future_sight: (i8, u8),
        force_switch: bool,
        force_trapped: bool,
        slow_uturn_move: bool,
        volatile_statuses: Vec<String>,
        substitute_health: i16,
        attack_boost: i8,
        defense_boost: i8,
        special_attack_boost: i8,
        special_defense_boost: i8,
        speed_boost: i8,
        accuracy_boost: i8,
        evasion_boost: i8,
        last_used_move: &str,
        switch_out_move_second_saved_move: &str,
    ) -> PyResult<Self> {
        let mut vs_hashset = HashSet::new();
        for vs in volatile_statuses {
            vs_hashset.insert(
                PokemonVolatileStatus::from_str(&vs).unwrap_or(PokemonVolatileStatus::NONE),
            );
        }

        pokemon.extend(std::iter::repeat(PyPokemon::create_fainted()).take(6 - pokemon.len()));

        Ok(Self {
            side: Side {
                active_index: PokemonIndex::from(active_index),
                baton_passing,
                pokemon: SidePokemon {
                    p0: pokemon[0].create_pokemon(),
                    p1: pokemon[1].create_pokemon(),
                    p2: pokemon[2].create_pokemon(),
                    p3: pokemon[3].create_pokemon(),
                    p4: pokemon[4].create_pokemon(),
                    p5: pokemon[5].create_pokemon(),
                },
                side_conditions: side_conditions.side_conditions,
                wish,
                future_sight: (future_sight.0, PokemonIndex::from(future_sight.1)),
                force_switch,
                force_trapped,
                slow_uturn_move,
                volatile_statuses: vs_hashset,
                substitute_health,
                attack_boost,
                defense_boost,
                special_attack_boost,
                special_defense_boost,
                speed_boost,
                accuracy_boost,
                evasion_boost,
                last_used_move: LastUsedMove::deserialize(last_used_move),
                damage_dealt: DamageDealt::default(),
                switch_out_move_second_saved_move: match Choices::from_str(
                    switch_out_move_second_saved_move,
                ) {
                    Ok(s) => s,
                    Err(()) => {
                        return Err(PyValueError::new_err(format!("Invalid switch_out_move_second_saved_move: {switch_out_move_second_saved_move:?}")))
                    }
                },
            },
        })
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self.side)
    }
}
