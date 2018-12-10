use std::fmt;

use crate::actions::Actions;
use crate::state::State;
use crate::world::World;

use crate::maxq::maxnode::{MaxNode, MaxNodeType};
use crate::maxq::nodestorage::NodeStorage;

use crate::maxq::MaxQParams;

#[derive(Debug, Clone, Copy)]
pub enum QNodeType {
    Get,
    NavigateForGet,
    PickUp,
    Put,
    NavigateForPut,
    DropOff,
    North(char),
    South(char),
    East(char),
    West(char),
}

#[derive(Debug, Clone, Copy)]
pub enum QChild {
    Primitive(usize),
    MaxNode(usize),
}

#[derive(Debug, Clone)]
pub struct QNode {
    node_type: QNodeType,
    completions: Vec<f64>,
    learning_completions: Vec<f64>,
}

impl QNode {
    pub fn new(node_type: QNodeType, world: &World) -> QNode {
        let num_taxi_states = (world.height as usize) * (world.width as usize);
        let num_fixed_positions = world.num_fixed_positions();

        let num_completions = match node_type {
            QNodeType::Get => num_fixed_positions * num_fixed_positions,
            QNodeType::NavigateForGet | QNodeType::NavigateForPut => num_fixed_positions,
            QNodeType::PickUp | QNodeType::DropOff => num_fixed_positions * num_taxi_states,
            QNodeType::Put => 0,
            QNodeType::North(_) | QNodeType::South(_) | QNodeType::East(_) | QNodeType::West(_) => {
                num_taxi_states
            }
        };

        QNode {
            node_type,
            completions: vec![0.0; num_completions],
            learning_completions: vec![0.0; num_completions],
        }
    }

    // returns (value, completion, leaf-action)
    pub fn evaluate(
        &self,
        nodes: &NodeStorage,
        world: &World,
        state: &State,
    ) -> Option<(f64, f64, Actions)> {
        let completion = self
            .get_completion_index(world, state)
            .map(|completion_index| self.completions[completion_index])
            .unwrap_or(0.0);

        let qchild = self.get_child(world, state)?;

        match qchild {
            QChild::Primitive(index) => {
                let primitive_node = &nodes.primitive_nodes[index];
                let (value, action) = primitive_node.evaluate(world, state);

                Some((value, completion, action))
            }

            QChild::MaxNode(index) => {
                let max_node = &nodes.max_nodes[index];

                max_node
                    .evaluate(nodes, world, state)
                    .map(|(value, _, action)| (value, completion, action))
            }
        }
    }

    pub fn evaluate_learning(
        &self,
        nodes: &NodeStorage,
        world: &World,
        state: &State,
    ) -> Option<(f64, f64, f64)> {
        let (learning_completion, completion) = self
            .get_completion_index(world, state)
            .map(|completion_index| {
                (
                    self.learning_completions[completion_index],
                    self.completions[completion_index],
                )
            })
            .unwrap_or((0.0, 0.0));

        let qchild = self.get_child(world, state)?;

        match qchild {
            QChild::Primitive(index) => {
                let primitive_node = &nodes.primitive_nodes[index];
                let (value, _) = primitive_node.evaluate(world, state);

                Some((value, learning_completion, completion))
            }

            QChild::MaxNode(index) => {
                let max_node = &nodes.max_nodes[index];

                max_node
                    .evaluate(nodes, world, state)
                    .map(|(value, _, _)| (value, learning_completion, completion))
            }
        }
    }

    pub fn update_learning_completion(
        &mut self,
        params: &MaxQParams,
        gamma: f64,
        result_learning_completion: f64,
        result_completion: f64,
        world: &World,
        state: &State,
    ) {
        if params.show_learning {
            println!("Updating completion for {}", self);
        }

        self.get_completion_index(world, state)
            .map(|completion_index| {
                let old_completion = self.completions[completion_index];

                self.learning_completions[completion_index] *= 1.0 - params.alpha;
                self.learning_completions[completion_index] +=
                    params.alpha * gamma * result_learning_completion;

                self.completions[completion_index] *= 1.0 - params.alpha;
                self.completions[completion_index] += params.alpha * gamma * result_completion;

                if params.show_learning {
                    println!(
                        "{} completion {} - was {} applied {} -> {}",
                        self,
                        completion_index,
                        old_completion,
                        result_completion,
                        self.completions[completion_index]
                    );
                }
            });
    }

    pub fn get_completion_index(&self, world: &World, state: &State) -> Option<usize> {
        match self.node_type {
            QNodeType::Get => passenger_state_index(world, state)
                .and_then(|index| add_destination_state_index(index, world, state)),

            QNodeType::NavigateForGet => passenger_state_index(world, state),

            QNodeType::PickUp => passenger_state_index(world, state)
                .and_then(|index| add_taxi_state_index(index, world, state)),

            QNodeType::Put => None,

            QNodeType::NavigateForPut => destination_state_index(world, state),

            QNodeType::DropOff => destination_state_index(world, state)
                .and_then(|index| add_taxi_state_index(index, world, state)),

            QNodeType::North(_) | QNodeType::South(_) | QNodeType::East(_) | QNodeType::West(_) => {
                taxi_state_index(world, state)
            }
        }
    }

    pub fn get_child(&self, world: &World, state: &State) -> Option<QChild> {
        match self.node_type {
            QNodeType::Get => Some(QChild::MaxNode(MaxNode::get_index(MaxNodeType::Get, world))),
            QNodeType::NavigateForGet => {
                let id = state.get_passenger()?;
                Some(QChild::MaxNode(MaxNode::get_index(
                    MaxNodeType::Navigate(id),
                    world,
                )))
            }
            QNodeType::PickUp => Some(QChild::Primitive(Actions::PickUp.to_index())),
            QNodeType::Put => Some(QChild::MaxNode(MaxNode::get_index(MaxNodeType::Put, world))),
            QNodeType::NavigateForPut => {
                let id = state.get_destination();
                Some(QChild::MaxNode(MaxNode::get_index(
                    MaxNodeType::Navigate(id),
                    world,
                )))
            }
            QNodeType::DropOff => Some(QChild::Primitive(Actions::DropOff.to_index())),

            QNodeType::North(_) => Some(QChild::Primitive(Actions::North.to_index())),
            QNodeType::South(_) => Some(QChild::Primitive(Actions::South.to_index())),
            QNodeType::East(_) => Some(QChild::Primitive(Actions::East.to_index())),
            QNodeType::West(_) => Some(QChild::Primitive(Actions::West.to_index())),
        }
    }

    pub fn build_nodes(world: &World) -> Vec<QNode> {
        let num_nodes = Self::num_nodes(world);
        let mut nodes = Vec::with_capacity(num_nodes);

        assert_eq!(nodes.len(), Self::get_index(QNodeType::Get, world));
        nodes.push(Self::new(QNodeType::Get, world));

        assert_eq!(
            nodes.len(),
            Self::get_index(QNodeType::NavigateForGet, world)
        );
        nodes.push(Self::new(QNodeType::NavigateForGet, world));

        assert_eq!(nodes.len(), Self::get_index(QNodeType::PickUp, world));
        nodes.push(Self::new(QNodeType::PickUp, world));

        assert_eq!(nodes.len(), Self::get_index(QNodeType::Put, world));
        nodes.push(Self::new(QNodeType::Put, world));

        assert_eq!(
            nodes.len(),
            Self::get_index(QNodeType::NavigateForPut, world)
        );
        nodes.push(Self::new(QNodeType::NavigateForPut, world));

        assert_eq!(nodes.len(), Self::get_index(QNodeType::DropOff, world));
        nodes.push(Self::new(QNodeType::DropOff, world));

        for id_index in 0..world.num_fixed_positions() {
            let id = world.get_fixed_id_from_index(id_index).unwrap();
            let node_type = QNodeType::North(id);

            assert_eq!(nodes.len(), Self::get_index(node_type, world));
            nodes.push(Self::new(node_type, world));
        }

        for id_index in 0..world.num_fixed_positions() {
            let id = world.get_fixed_id_from_index(id_index).unwrap();
            let node_type = QNodeType::South(id);

            assert_eq!(nodes.len(), Self::get_index(node_type, world));
            nodes.push(Self::new(node_type, world));
        }

        for id_index in 0..world.num_fixed_positions() {
            let id = world.get_fixed_id_from_index(id_index).unwrap();
            let node_type = QNodeType::East(id);

            assert_eq!(nodes.len(), Self::get_index(node_type, world));
            nodes.push(Self::new(node_type, world));
        }

        for id_index in 0..world.num_fixed_positions() {
            let id = world.get_fixed_id_from_index(id_index).unwrap();
            let node_type = QNodeType::West(id);

            assert_eq!(nodes.len(), Self::get_index(node_type, world));
            nodes.push(Self::new(node_type, world));
        }
        nodes
    }

    pub fn get_index(node_type: QNodeType, world: &World) -> usize {
        match node_type {
            QNodeType::Get => 0,
            QNodeType::NavigateForGet => 1,
            QNodeType::PickUp => 2,
            QNodeType::Put => 3,
            QNodeType::NavigateForPut => 4,
            QNodeType::DropOff => 5,
            QNodeType::North(id) => {
                let id_index = world.get_fixed_index(id).unwrap();
                6 + id_index
            }
            QNodeType::South(id) => {
                let id_index = world.get_fixed_index(id).unwrap();
                6 + world.num_fixed_positions() + id_index
            }
            QNodeType::East(id) => {
                let id_index = world.get_fixed_index(id).unwrap();
                6 + 2 * world.num_fixed_positions() + id_index
            }
            QNodeType::West(id) => {
                let id_index = world.get_fixed_index(id).unwrap();
                6 + 3 * world.num_fixed_positions() + id_index
            }
        }
    }

    fn num_nodes(world: &World) -> usize {
        6 + 4 * world.num_fixed_positions()
    }
}

impl fmt::Display for QNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type {
            QNodeType::Get => write!(f, "Get"),
            QNodeType::NavigateForGet => write!(f, "NavigateForGet"),
            QNodeType::PickUp => write!(f, "Pickup"),
            QNodeType::Put => write!(f, "Put"),
            QNodeType::NavigateForPut => write!(f, "NavigateForPut"),
            QNodeType::DropOff => write!(f, "DropOff"),
            QNodeType::North(id) => write!(f, "North({})", id),
            QNodeType::South(id) => write!(f, "South({})", id),
            QNodeType::East(id) => write!(f, "East({})", id),
            QNodeType::West(id) => write!(f, "West({})", id),
        }
    }
}

fn passenger_state_index(world: &World, state: &State) -> Option<usize> {
    state
        .get_passenger()
        .and_then(|passenger_id| world.get_fixed_index(passenger_id))
}

// fn add_passenger_state_index(mut index: usize, world: &World, state: &State) -> Option<usize> {
//     passenger_state_index(world, state).map(|passenger_index| {
//         index *= world.num_fixed_positions();
//         index += passenger_index;
//         index
//     })
// }

fn destination_state_index(world: &World, state: &State) -> Option<usize> {
    world.get_fixed_index(state.get_destination())
}

fn add_destination_state_index(mut index: usize, world: &World, state: &State) -> Option<usize> {
    destination_state_index(world, state).map(|destination_index| {
        index *= world.num_fixed_positions();
        index += destination_index;
        index
    })
}

fn taxi_state_index(world: &World, state: &State) -> Option<usize> {
    let mut index = state.get_taxi().y as usize;
    index *= world.width as usize;
    index += state.get_taxi().x as usize;
    Some(index)
}

fn add_taxi_state_index(mut index: usize, world: &World, state: &State) -> Option<usize> {
    taxi_state_index(world, state).map(|taxi_index| {
        index *= (world.height * world.width) as usize;
        index += taxi_index;
        index
    })
}
