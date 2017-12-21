use std::fmt;
use std::slice::Iter;

use rand::Rng;

use state::State;
use actions::Actions;
use world::World;

use maxq::qnode::{QNode, QNodeType};
use maxq::nodestorage::NodeStorage;
use maxq::MaxQParams;

#[derive(Debug, Clone, Copy)]
pub enum MaxNodeType {
    Root,
    Get,
    Put,
    Navigate(char),
}

#[derive(Debug, Clone)]
pub struct MaxNode {
    node_type: MaxNodeType,
    qnodes: Vec<usize>,
}

impl MaxNode {
    pub fn evaluate(
        &self,
        nodes: &NodeStorage,
        world: &World,
        state: &State,
    ) -> Option<(f64, usize, Actions)> {
        let mut result = None;

        if !self.terminal_state(world, state) {
            let mut highest_q = None;

            for qnode_index in &self.qnodes {
                if let Some((value, completion, action)) =
                    nodes.q_nodes[*qnode_index].evaluate(nodes, world, state)
                {
                    let q = Some(value + completion);

                    if q > highest_q {
                        highest_q = q;

                        result = Some((value, *qnode_index, action));
                    }
                }
            }
        }

        result
    }

    pub fn result_state_values(
        &self,
        nodes: &NodeStorage,
        world: &World,
        state: &State,
    ) -> Option<(f64, f64)> {
        let mut result = None;

        if !self.terminal_state(world, state) {
            let mut highest_q = None;

            for qnode_index in &self.qnodes {
                if let Some((value, learning_completion, completion)) =
                    nodes.q_nodes[*qnode_index].evaluate_learning(nodes, world, state)
                {
                    let q = Some(value + learning_completion);

                    if q > highest_q {
                        highest_q = q;

                        result = Some((value + learning_completion, value + completion));
                    }
                }
            }
        }

        result
    }

    pub fn learning_reward(&self, _world: &World, state: &State) -> f64 {
        match self.node_type {
            MaxNodeType::Put => if state.at_destination() {
                0.0
            } else {
                -100.0
            },
            _ => 0.0,
        }
    }

    pub fn select_child_to_learn<R: Rng>(
        &self,
        nodes: &NodeStorage,
        params: &MaxQParams,
        world: &World,
        state: &State,
        rng: &mut R,
    ) -> Option<usize> {
        let nongreedy_roll = rng.gen_range(0.0f64, 1.0f64);

        if nongreedy_roll < params.epsilon {
            rng.choose(&self.qnodes).map(|child_index| *child_index)
        } else {
            self.evaluate(nodes, world, state)
                .map(|(_, child_index, _)| child_index)
        }
    }

    pub fn terminal_state(&self, world: &World, state: &State) -> bool {
        match self.node_type {
            MaxNodeType::Root => state.at_destination(),
            MaxNodeType::Get => state.get_passenger() == None,
            MaxNodeType::Put => state.get_passenger() != None,
            MaxNodeType::Navigate(id) => Some(state.get_taxi()) == world.get_fixed_position(id),
        }
    }

    pub fn build_nodes(world: &World) -> Vec<MaxNode> {
        let num_nodes = Self::num_nodes(world);
        let mut nodes = Vec::with_capacity(num_nodes);

        {
            let node_type = MaxNodeType::Root;
            let qnodes = vec![
                QNode::get_index(QNodeType::Get, world),
                QNode::get_index(QNodeType::Put, world),
            ];

            assert_eq!(nodes.len(), Self::get_index(node_type, world));

            nodes.push(MaxNode { node_type, qnodes });
        }

        {
            let node_type = MaxNodeType::Get;

            let qnodes = vec![
                QNode::get_index(QNodeType::PickUp, world),
                QNode::get_index(QNodeType::NavigateForGet, world),
            ];

            assert_eq!(nodes.len(), Self::get_index(node_type, world));

            nodes.push(MaxNode { node_type, qnodes });
        }

        {
            let node_type = MaxNodeType::Put;

            let qnodes = vec![
                QNode::get_index(QNodeType::DropOff, world),
                QNode::get_index(QNodeType::NavigateForPut, world),
            ];

            assert_eq!(nodes.len(), Self::get_index(node_type, world));

            nodes.push(MaxNode { node_type, qnodes });
        }

        for id_index in 0..world.num_fixed_positions() {
            let id = world.get_fixed_id_from_index(id_index).unwrap();

            let node_type = MaxNodeType::Navigate(id);

            let qnodes = vec![
                QNode::get_index(QNodeType::North(id), world),
                QNode::get_index(QNodeType::South(id), world),
                QNode::get_index(QNodeType::East(id), world),
                QNode::get_index(QNodeType::West(id), world),
            ];

            assert_eq!(nodes.len(), Self::get_index(node_type, world));

            nodes.push(MaxNode { node_type, qnodes });
        }

        assert_eq!(nodes.len(), num_nodes);

        nodes
    }

    pub fn get_index(node_type: MaxNodeType, world: &World) -> usize {
        match node_type {
            MaxNodeType::Root => 0,
            MaxNodeType::Get => 1,
            MaxNodeType::Put => 2,
            MaxNodeType::Navigate(id) => {
                let id_index = world.get_fixed_index(id).unwrap();
                3 + id_index
            }
        }
    }

    fn num_nodes(world: &World) -> usize {
        3 + world.num_fixed_positions()
    }

    pub fn qnode_index_iter(&self) -> Iter<usize> {
        self.qnodes.iter()
    }
}

impl fmt::Display for MaxNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type {
            MaxNodeType::Root => write!(f, "Root"),
            MaxNodeType::Get => write!(f, "Get"),
            MaxNodeType::Put => write!(f, "Put"),
            MaxNodeType::Navigate(id) => write!(f, "Navigate({})", id),
        }
    }
}
