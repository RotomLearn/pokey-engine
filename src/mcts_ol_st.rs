use poke_engine::{
    evaluate::evaluate,
    generate_instructions::generate_instructions_from_move_pair,
    instruction::StateInstructions,
    pokemon::PokemonName,
    state::{MoveChoice, State},
};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

fn sigmoid(x: f32) -> f32 {
    // Tuned so that ~200 points is very close to 1.0
    1.0 / (1.0 + (-0.0125 * x).exp())
}

// Thread-local RNG
thread_local! {
    static THREAD_RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct UniqueMove {
    move_choice: MoveChoice,
    pokemon_name: PokemonName,
    is_switch: bool,
}

pub struct OpponentMoveStats {
    pub visits: i64,
    pub value: f32,
    pub last_ucb: Option<f32>,
}

#[derive(Clone)]
pub struct MoveHistoryEntry {
    opp_move: MoveChoice,
    opp_active: PokemonName,
}

pub struct MCTS {
    root: Rc<RefCell<MCTSNode>>,
    max_depth_seen: Rc<RefCell<usize>>,
}

pub struct MCTSNode {
    pub parent: Option<Weak<RefCell<MCTSNode>>>,
    pub children: HashMap<MoveChoice, Rc<RefCell<MCTSNode>>>,
    pub opponent_move_stats: HashMap<UniqueMove, OpponentMoveStats>,
    pub visits: i64,
    pub value: f32,
    pub our_move: Option<MoveChoice>,
    pub depth: i32,
    pub last_simulation_score: Option<f32>,
    pub actual_opponent_move: Option<UniqueMove>,
    pub original_active: Option<PokemonName>,
}

impl MCTS {
    pub fn new() -> Self {
        MCTS {
            root: Rc::new(RefCell::new(MCTSNode::new(0))),
            max_depth_seen: Rc::new(RefCell::new(0)),
        }
    }
}

impl MCTSNode {
    pub fn new(depth: i32) -> Self {
        MCTSNode {
            parent: None,
            children: HashMap::new(),
            opponent_move_stats: HashMap::new(),
            visits: 0,
            value: 0.0,
            our_move: None,
            depth,
            last_simulation_score: None,
            actual_opponent_move: None,
            original_active: None,
        }
    }

    #[inline(always)]
    pub fn ucb1_score(&self, parent_visits: i64) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let visits_f = self.visits as f32;
        let exploit = self.value / visits_f;
        let explore = (2.0 * (parent_visits as f32).ln() / visits_f).sqrt();
        exploit + explore
    }

    pub fn select_opponent_move(
        &mut self,
        available_moves: &[MoveChoice],
        state: &State,
    ) -> MoveChoice {
        let current_opponent_active = state.side_two.get_active_immutable().id;

        // Identify untried moves
        let untried_moves: Vec<_> = available_moves
            .iter()
            .filter(|m| {
                let unique_move = UniqueMove {
                    move_choice: (*m).clone(),
                    pokemon_name: current_opponent_active,
                    is_switch: matches!(m, MoveChoice::Switch(_)),
                };
                !self.opponent_move_stats.contains_key(&unique_move)
            })
            .collect();

        if !untried_moves.is_empty() {
            let chosen = untried_moves[thread_rng().gen_range(0..untried_moves.len())].clone();
            let unique_move = UniqueMove {
                move_choice: chosen.clone(),
                pokemon_name: current_opponent_active,
                is_switch: matches!(chosen, MoveChoice::Switch(_)),
            };

            self.opponent_move_stats.insert(
                unique_move,
                OpponentMoveStats {
                    visits: 0,
                    value: 0.0,
                    last_ucb: None,
                },
            );

            return chosen;
        }

        // Calculate UCB1 scores
        let total_visits = self
            .opponent_move_stats
            .values()
            .map(|stats| stats.visits)
            .sum::<i64>();

        let mut best_score = f32::NEG_INFINITY;
        let mut best_move = None;

        for move_choice in available_moves {
            let unique_move = UniqueMove {
                move_choice: move_choice.clone(),
                pokemon_name: current_opponent_active,
                is_switch: matches!(move_choice, MoveChoice::Switch(_)),
            };

            if let Some(stats) = self.opponent_move_stats.get_mut(&unique_move) {
                let exploitation = 1.0 - (stats.value / stats.visits as f32);
                let exploration = (2.0 * (total_visits as f32).ln() / stats.visits as f32).sqrt();
                let ucb_score = exploitation + exploration;
                stats.last_ucb = Some(ucb_score);

                if ucb_score > best_score {
                    best_score = ucb_score;
                    best_move = Some(move_choice.clone());
                }
            }
        }

        best_move.unwrap_or_else(|| available_moves[0].clone())
    }

    fn select_and_expand(
        node: Rc<RefCell<MCTSNode>>,
        state: &mut State,
        max_depth_seen: &Rc<RefCell<usize>>,
    ) -> (Rc<RefCell<MCTSNode>>, SmallVec<[MoveHistoryEntry; 16]>) {
        let mut current_node = node;
        let mut move_history = SmallVec::new();

        loop {
            {
                let node_guard = current_node.borrow();
                let mut depth_seen = max_depth_seen.borrow_mut();
                *depth_seen = (*depth_seen).max(node_guard.depth as usize);
            }

            let (our_moves, opp_moves) = state.get_all_options();
            if state.battle_is_over() != 0.0 || (our_moves.is_empty() && opp_moves.is_empty()) {
                return (current_node, move_history);
            }

            let valid_our_moves = if our_moves.contains(&MoveChoice::None) {
                vec![MoveChoice::None]
            } else {
                our_moves
                    .iter()
                    .filter(|&m| !matches!(m, MoveChoice::None))
                    .cloned()
                    .collect::<Vec<_>>()
            };

            let valid_opp_moves = if opp_moves.contains(&MoveChoice::None) {
                let switch_moves: Vec<_> = opp_moves
                    .iter()
                    .filter(|&m| matches!(m, MoveChoice::Switch(_)))
                    .cloned()
                    .collect();
                if !switch_moves.is_empty() {
                    switch_moves
                } else {
                    vec![MoveChoice::None]
                }
            } else if our_moves.iter().all(|m| matches!(m, MoveChoice::Switch(_))) {
                vec![MoveChoice::None]
            } else {
                opp_moves
                    .iter()
                    .filter(|&m| !matches!(m, MoveChoice::None))
                    .cloned()
                    .collect::<Vec<_>>()
            };

            // Check for untried moves
            let untried_move = {
                let node_guard = current_node.borrow();
                valid_our_moves
                    .iter()
                    .find(|m| !node_guard.children.contains_key(*m))
                    .cloned()
            };

            if let Some(our_move) = untried_move {
                let current_opponent_active = state.side_two.get_active_immutable().id;
                let current_our_active = state.side_one.get_active_immutable().id;

                let opp_move = if valid_opp_moves
                    .iter()
                    .all(|m| matches!(m, MoveChoice::Switch(_)))
                {
                    valid_opp_moves[0].clone()
                } else {
                    current_node
                        .borrow_mut()
                        .select_opponent_move(&valid_opp_moves, state)
                };

                let unique_move = UniqueMove {
                    move_choice: opp_move.clone(),
                    pokemon_name: current_opponent_active,
                    is_switch: matches!(opp_move, MoveChoice::Switch(_)),
                };

                let instructions =
                    generate_instructions_from_move_pair(state, &our_move, &opp_move, true);
                let chosen_inst = sample_instruction(&instructions);
                state.apply_instructions(&chosen_inst.instruction_list);

                move_history.push(MoveHistoryEntry {
                    opp_move: opp_move.clone(),
                    opp_active: current_opponent_active,
                });

                let new_depth = current_node.borrow().depth + 1;
                let mut new_node = MCTSNode::new(new_depth);
                new_node.our_move = Some(our_move.clone());
                new_node.parent = Some(Rc::downgrade(&current_node));
                new_node.actual_opponent_move = Some(unique_move);
                new_node.original_active = Some(current_our_active);

                let new_node_rc = Rc::new(RefCell::new(new_node));
                current_node
                    .borrow_mut()
                    .children
                    .insert(our_move, new_node_rc.clone());

                return (new_node_rc, move_history);
            }

            // Selection phase
            let selection_result = {
                let node_guard = current_node.borrow();
                let mut best_move = None;
                let mut best_score = f32::NEG_INFINITY;
                let mut best_node = None;

                for (move_choice, child) in &node_guard.children {
                    if valid_our_moves.contains(move_choice) {
                        let child_guard = child.borrow();
                        let score = child_guard.ucb1_score(node_guard.visits);

                        if score > best_score {
                            best_score = score;
                            best_move = Some(move_choice.clone());
                            best_node = Some(Rc::clone(child));
                        }
                    }
                }
                (best_move, best_node)
            };

            let (selected_move, next_node) = match selection_result {
                (Some(mov), Some(node)) => (mov, node),
                _ => return (current_node, move_history),
            };

            let selected_opp_move = if valid_opp_moves
                .iter()
                .all(|m| matches!(m, MoveChoice::Switch(_)))
            {
                valid_opp_moves[0].clone()
            } else {
                current_node
                    .borrow_mut()
                    .select_opponent_move(&valid_opp_moves, state)
            };

            let current_opponent_active = state.side_two.get_active_immutable().id;
            let unique_move = UniqueMove {
                move_choice: selected_opp_move.clone(),
                pokemon_name: current_opponent_active,
                is_switch: matches!(selected_opp_move, MoveChoice::Switch(_)),
            };

            let instructions = generate_instructions_from_move_pair(
                state,
                &selected_move,
                &selected_opp_move,
                true,
            );
            let chosen_inst = sample_instruction(&instructions);
            state.apply_instructions(&chosen_inst.instruction_list);

            move_history.push(MoveHistoryEntry {
                opp_move: selected_opp_move,
                opp_active: current_opponent_active,
            });

            next_node.borrow_mut().actual_opponent_move = Some(unique_move);
            current_node = next_node;
        }
    }
    fn backpropagate(node: Rc<RefCell<MCTSNode>>, score: f32, move_history: &[MoveHistoryEntry]) {
        let mut current = node;

        // Update the leaf node
        {
            let mut node_guard = current.borrow_mut();
            node_guard.visits += 1;
            node_guard.value += score;
            node_guard.last_simulation_score = Some(score);
        }

        // Walk back up the tree
        for entry in move_history.iter().rev() {
            let parent = {
                let node_guard = current.borrow();
                node_guard.parent.as_ref().and_then(|p| p.upgrade())
            };

            if let Some(parent_node) = parent {
                {
                    let mut parent_guard = parent_node.borrow_mut();
                    parent_guard.visits += 1;
                    parent_guard.value += score;

                    let unique_move = UniqueMove {
                        move_choice: entry.opp_move.clone(),
                        pokemon_name: entry.opp_active,
                        is_switch: matches!(entry.opp_move, MoveChoice::Switch(_)),
                    };

                    let stats = parent_guard
                        .opponent_move_stats
                        .entry(unique_move)
                        .or_insert(OpponentMoveStats {
                            visits: 0,
                            value: 0.0,
                            last_ucb: None,
                        });
                    stats.visits += 1;
                    stats.value += score;
                }
                current = parent_node;
            } else {
                break;
            }
        }
    }
}

fn sample_instruction(instructions: &[StateInstructions]) -> &StateInstructions {
    if instructions.len() == 1 {
        return &instructions[0];
    }

    let mut weights = Vec::with_capacity(instructions.len());
    weights.extend(instructions.iter().map(|i| i.percentage as f64));

    THREAD_RNG.with(|rng| match WeightedIndex::new(&weights) {
        Ok(dist) => &instructions[dist.sample(&mut *rng.borrow_mut())],
        Err(_) => &instructions[0],
    })
}

pub fn perform_mcts_search_st(
    state: &mut State,
    iterations: Option<u32>,
    time_limit: Option<Duration>,
) -> (Vec<(String, f32)>, i64) {
    let start_time = Instant::now();
    let mcts = MCTS::new();
    let root_eval = evaluate(state);

    while !should_stop(&start_time, iterations, time_limit, &mcts) {
        let mut sim_state = state.clone();
        let (selected_node, move_history) = MCTSNode::select_and_expand(
            Rc::clone(&mcts.root),
            &mut sim_state,
            &mcts.max_depth_seen,
        );

        let score = if sim_state.battle_is_over() != 0.0 {
            if sim_state.battle_is_over() > 0.0 {
                1.0
            } else {
                0.0
            }
        } else {
            sigmoid(evaluate(&sim_state) - root_eval)
        };

        MCTSNode::backpropagate(selected_node, score, &move_history);
    }
    let root = mcts.root.borrow();

    choose_best_move(&root, state)
}

fn choose_best_move(root: &MCTSNode, state: &State) -> (Vec<(String, f32)>, i64) {
    let (our_moves, _) = state.get_all_options();
    let mut combined_stats: HashMap<MoveChoice, (i64, f32)> = HashMap::new();

    // Collect statistics
    for mov in &our_moves {
        if let Some(child) = root.children.get(mov) {
            let child_ref = child.borrow();
            let entry = combined_stats.entry(mov.clone()).or_insert((0, 0.0));
            entry.0 += child_ref.visits;
            entry.1 += child_ref.value;
        }
    }

    // Choose best move based on visit count
    let mut policy = Vec::new();

    for (mov, (visits, score)) in &combined_stats {
        policy.push((
            mov.to_string(&state.side_one),
            *visits as f32 / root.visits as f32,
        ));
    }

    (policy, root.visits)
}

fn should_stop(
    start_time: &Instant,
    iterations: Option<u32>,
    time_limit: Option<Duration>,
    mcts: &MCTS,
) -> bool {
    let visits = mcts.root.borrow().visits;

    if let Some(max_iter) = iterations {
        if visits >= max_iter as i64 {
            return true;
        }
    }

    if let Some(limit) = time_limit {
        if start_time.elapsed() >= limit {
            return true;
        }
    }

    visits >= 10_000_000
}
