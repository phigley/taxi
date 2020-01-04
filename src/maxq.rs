mod maxnode;
mod nodestorage;
mod primitivenode;
mod qnode;

use rand::Rng;

use crate::actions::Actions;
use crate::state::State;
use crate::state_indexer::StateIndexer;
use crate::world::World;

use crate::runner::{Attempt, Runner};

use self::nodestorage::NodeStorage;
use self::qnode::QChild;

#[derive(Debug, Clone, Copy)]
pub struct MaxQParams {
    alpha: f64,
    gamma: f64,
    epsilon: f64,

    show_learning: bool,
}

#[derive(Debug, Clone)]
pub struct MaxQ {
    params: MaxQParams,
    nodes: NodeStorage,
}

impl MaxQ {
    pub fn new(world: &World, alpha: f64, gamma: f64, epsilon: f64, show_learning: bool) -> MaxQ {
        let initial_q_value = if gamma < 1.0 {
            world.max_reward() / (1.0 - gamma)
        } else {
            world.max_reward()
        };

        let nodes = NodeStorage::new(initial_q_value, world);

        let params = MaxQParams {
            alpha,
            gamma,
            epsilon,

            show_learning,
        };

        MaxQ { params, nodes }
    }

    fn evaluate(&self, world: &World, state: &State) -> Option<Actions> {
        self.nodes.max_nodes[0]
            .evaluate(&self.nodes, world, state)
            .map(|(_, _, action)| action)
    }

    fn maxq_apply_selection<R: Rng>(
        &mut self,
        qchild: QChild,
        world: &World,
        state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Option<(State, Vec<State>)> {
        match qchild {
            QChild::Primitive(child_primitive_index) => {
                let primitive_node = &mut self.nodes.primitive_nodes[child_primitive_index];

                let (reward, next_state) = state.apply_action(world, primitive_node.get_action());

                primitive_node.apply_experience(&self.params, world, &state, reward, &next_state);

                Some((next_state, vec![state]))
            }

            QChild::MaxNode(child_max_index) => {
                self.maxq_q(child_max_index, world, state, max_steps, rng)
            }
        }
    }

    fn maxq_q<R: Rng>(
        &mut self,
        max_index: usize,
        world: &World,
        mut state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Option<(State, Vec<State>)> {
        let mut seq = Vec::new();

        while !self.nodes.max_nodes[max_index].terminal_state(world, &state)
            && seq.len() < max_steps
        {
            if self.params.show_learning {
                println!(
                    "step {}/{} node {} - {}\n{}",
                    seq.len(),
                    max_steps,
                    max_index,
                    self.nodes.max_nodes[max_index],
                    state.display(world)
                );
            }

            let child_q_index = self.nodes.max_nodes[max_index].select_child_to_learn(
                &self.nodes,
                &self.params,
                world,
                &state,
                rng,
            )?;

            let qchild = self.nodes.q_nodes[child_q_index].get_child(world, &state)?;

            if self.params.show_learning {
                println!(
                    "Node {} selected child {} - {}",
                    self.nodes.max_nodes[max_index],
                    child_q_index,
                    self.nodes.q_nodes[child_q_index]
                );
            }

            let (next_state, mut child_seq) =
                self.maxq_apply_selection(qchild, world, state, max_steps - seq.len(), rng)?;

            // A terminal state check should be run for all parents here.
            // For taxi, there is no way for a parent to terminate
            // without the current node terminating, so not needed here.

            let child_completed = match qchild {
                QChild::Primitive(_) => true,
                QChild::MaxNode(child_max_index) => {
                    self.nodes.max_nodes[child_max_index].terminal_state(world, &next_state)
                }
            };

            if child_completed {
                let learning_reward =
                    self.nodes.max_nodes[max_index].learning_reward(world, &next_state);

                if self.params.show_learning {
                    println!(
                        "maxq_q {} - {} for state:\n{}",
                        max_index,
                        self.nodes.max_nodes[max_index],
                        next_state.display(world),
                    );
                }

                let (result_state_learning_value, result_state_value) = self.nodes.max_nodes
                    [max_index]
                    .result_state_values(&self.nodes, world, &next_state)
                    .unwrap_or((0.0, 0.0));

                let mut accum_gamma = self.params.gamma;
                for child_state in child_seq.iter().rev() {
                    self.nodes.q_nodes[child_q_index].update_learning_completion(
                        &self.params,
                        accum_gamma,
                        learning_reward + result_state_learning_value,
                        result_state_value,
                        world,
                        child_state,
                    );
                    accum_gamma *= self.params.gamma;
                }
            }

            seq.append(&mut child_seq);
            state = next_state;
        }

        if self.params.show_learning {
            println!(
                "Step {}/{} terminating node {} - {}",
                seq.len(),
                max_steps,
                max_index,
                self.nodes.max_nodes[max_index]
            );
        }
        Some((state, seq))
    }
}

impl Runner for MaxQ {
    fn learn<R: Rng>(
        &mut self,
        world: &World,
        state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Option<usize> {
        if self.params.show_learning {
            println!("Learning:\n{:#?}\n{}\n", state, state.display(world));
        }

        let (final_state, seq) = self.maxq_q(0, world, state, max_steps, rng)?;

        if self.params.show_learning {
            println!(
                "Finished {} steps:\n{:#?}\n{}\n{}",
                seq.len(),
                final_state,
                final_state.display(world),
                if final_state.at_destination() {
                    "success"
                } else {
                    "failed"
                }
            );
        }
        if final_state.at_destination() {
            Some(seq.len())
        } else {
            None
        }
    }

    fn attempt<R: Rng>(
        &self,
        world: &World,
        mut state: State,
        max_steps: usize,
        mut _rng: &mut R,
    ) -> Attempt {
        let mut attempt = Attempt::new(state, max_steps);

        for _ in 0..max_steps {
            if state.at_destination() {
                break;
            }

            if let Some(next_action) = self.evaluate(world, &state) {
                attempt.step(next_action);
                let (_, next_state) = state.apply_action(world, next_action);
                state = next_state;
            } else {
                break;
            }
        }

        if state.at_destination() {
            attempt.succeeded()
        }

        attempt
    }

    fn solves<R: Rng>(
        &self,
        world: &World,
        mut state: State,
        max_steps: usize,
        mut _rng: &mut R,
    ) -> bool {
        for _ in 0..max_steps {
            if state.at_destination() {
                return true;
            }

            if let Some(next_action) = self.evaluate(world, &state) {
                let (_, next_state) = state.apply_action(world, next_action);
                state = next_state;
            } else {
                break;
            }
        }

        state.at_destination()
    }

    fn report_training_result(&self, world: &World, _steps: Option<usize>) {
        let state_indexer = StateIndexer::new(world);

        for si in 0..state_indexer.num_states() {
            if let Some(state) = state_indexer.get_state(world, si) {
                if !state.at_destination() {
                    println!("{}\n{}", si, state.display(world));
                    if let Some(action) = self.evaluate(world, &state) {
                        println!("Result {}", action,);

                        let mut current_max_index = 0;
                        loop {
                            if let Some((_, child_index, _)) = self.nodes.max_nodes
                                [current_max_index]
                                .evaluate(&self.nodes, world, &state)
                            {
                                println!(
                                    "{} chose {}",
                                    self.nodes.max_nodes[current_max_index],
                                    self.nodes.q_nodes[child_index]
                                );
                                if let Some(qchild) =
                                    self.nodes.q_nodes[child_index].get_child(world, &state)
                                {
                                    match qchild {
                                        QChild::Primitive(_) => break,
                                        QChild::MaxNode(max_index) => {
                                            current_max_index = max_index;
                                        }
                                    }
                                } else {
                                    println!("Failed to find child for index {}!", child_index);
                                    break;
                                }
                            } else {
                                println!("Failed to evaluate");
                                break;
                            }
                        }
                    } else {
                        println!("Failed to evaluate!");
                    }

                    for (max_node_index, max_node) in self.nodes.max_nodes.iter().enumerate() {
                        println!("{} {} :", max_node_index, max_node);

                        for q_index in max_node.qnode_index_iter() {
                            let qnode = &self.nodes.q_nodes[*q_index];

                            if let Some((value, completion, _)) =
                                qnode.evaluate(&self.nodes, world, &state)
                            {
                                let completion_index = qnode.get_completion_index(world, &state);

                                println!(
                                    "  {} {} {:?} => {} + {} = {}",
                                    q_index,
                                    qnode,
                                    completion_index,
                                    value,
                                    completion,
                                    value + completion
                                );
                            } else {
                                println!("  {} {} => does not evaluate", q_index, qnode,);
                            }
                        }
                    }

                    println!("\n");
                }
            }
        }
        // println!("{:#?}", self.max_nodes);
    }
}
