use crate::world::World;

use crate::maxq::maxnode::MaxNode;
use crate::maxq::primitivenode::PrimitiveNode;
use crate::maxq::qnode::QNode;

#[derive(Debug, Clone)]
pub struct NodeStorage {
    pub max_nodes: Vec<MaxNode>,
    pub q_nodes: Vec<QNode>,
    pub primitive_nodes: Vec<PrimitiveNode>,
}

impl NodeStorage {
    pub fn new(initial_q_value: f64, world: &World) -> NodeStorage {
        let max_nodes = MaxNode::build_nodes(world);
        let q_nodes = QNode::build_nodes(world);
        let primitive_nodes = PrimitiveNode::build_nodes(initial_q_value);

        NodeStorage {
            max_nodes,
            q_nodes,
            primitive_nodes,
        }
    }
}
