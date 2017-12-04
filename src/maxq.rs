

use rand::Rng;

use state::State;
use actions::Actions;
use world::World;
use state_indexer::StateIndexer;

use runner::{Runner, Attempt};


#[derive(Debug, Clone)]
struct PrimitiveNode {
    action: Actions,
    values: Vec<f64>, // indexed by state_index
}

// impl PrimitiveNode {
//     fn new(action: Actions, initial_q_value: f64) -> PrimitiveNode {
//         let num_values = match action {
//             Actions::PickUp | Actions::DropOff => 2,
//             Actions::North | Actions::South | Actions::East | Actions::West => 1,
//         };

//         PrimitiveNode {
//             action,
//             values: vec![initial_q_value; num_values],
//         }
//     }

//     fn get_index(&self, world: &World, state: &State) -> usize {
//         match self.action {
//             Actions::PickUp => {
//                 match state.get_passenger() {
//                     Some(id) if world.get_fixed_position(id) == Some(state.get_taxi()) => 0,
//                     _ => 1,
//                 }
//             }
//             Actions::DropOff => {
//                 match world.get_fixed_id(state.get_taxi()) {
//                     None if state.get_passenger() == None => 0,
//                     _ => 1,
//                 }
//             }
//             Actions::North | Actions::South | Actions::East | Actions::West => 0,
//         }
//     }
// }

#[derive(Debug, Clone, Copy)]
enum CompoundNodeType {
    Root,
    Get,
    Put,
    Navigate(char),
}

#[derive(Debug, Clone)]
struct CompoundNode {
    compound_type: CompoundNodeType,
    children: Vec<usize>,
    completion: Vec<f64>, // indexed by state_index*num_nodes + node_index
    learning_completion: Vec<f64>,
}

#[derive(Debug, Clone)]
enum MaxNode {
    Primitive(PrimitiveNode),
    Compound(CompoundNode),
}


#[derive(Debug, Clone)]
pub struct MaxQ {
    alpha: f64,
    gamma: f64,
    epsilon: f64,
    show_table: bool,

    state_indexer: StateIndexer,

    max_nodes: Vec<MaxNode>,
    max_node_visits: Vec<usize>,

    start_actions: usize,
}

impl MaxQ {
    pub fn new(world: &World, alpha: f64, gamma: f64, epsilon: f64, show_table: bool) -> MaxQ {

        let state_indexer = StateIndexer::new(world);
        let num_states = state_indexer.num_states();

        let num_navigates = world.num_fixed_positions();

        let start_get = 1;
        let start_put = start_get + 1;
        let start_navigates = start_put + 1; // put + drop-off
        let start_actions = start_navigates + num_navigates;

        let num_nodes = start_actions + Actions::NUM_ELEMENTS;

        let mut max_nodes = Vec::with_capacity(num_nodes);


        let root_node = CompoundNode {
            compound_type: CompoundNodeType::Root,
            children: vec![start_get, start_put],
            completion: vec![0.0; num_states * num_nodes],
            learning_completion: vec![0.0; num_states * num_nodes],
        };

        max_nodes.push(MaxNode::Compound(root_node));

        assert!(max_nodes.len() == start_get);
        let get_node = {

            let num_children = 1 + num_navigates;
            let mut children = Vec::with_capacity(num_children);

            for i in 0..num_navigates {
                children.push(start_navigates + i);
            }

            children.push(start_actions + Actions::PickUp.to_index());

            CompoundNode {
                compound_type: CompoundNodeType::Get,
                children,
                completion: vec![0.0; num_states * num_nodes],
                learning_completion: vec![0.0; num_states * num_nodes],
            }
        };
        max_nodes.push(MaxNode::Compound(get_node));


        assert!(max_nodes.len() == start_put);
        let put_node = {
            let num_children = 1 + num_navigates;
            let mut children = Vec::with_capacity(num_children);
            children.push(start_actions + Actions::DropOff.to_index());

            for i in 0..num_navigates {
                children.push(start_navigates + i);
            }

            CompoundNode {
                compound_type: CompoundNodeType::Put,
                children,
                completion: vec![0.0; num_states * num_nodes],
                learning_completion: vec![0.0; num_states * num_nodes],
            }
        };
        max_nodes.push(MaxNode::Compound(put_node));

        assert!(max_nodes.len() == start_navigates);
        let movement_action_indices = vec![
            start_actions + Actions::North.to_index(),
            start_actions + Actions::South.to_index(),
            start_actions + Actions::East.to_index(),
            start_actions + Actions::West.to_index(),
        ];

        for fixed_position_index in 0..world.num_fixed_positions() {
            let id = world.get_fixed_id_from_index(fixed_position_index).unwrap();

            let children = movement_action_indices.clone();

            let navigate_node = CompoundNode {
                compound_type: CompoundNodeType::Navigate(id),
                children,
                completion: vec![0.0; num_states * num_nodes],
                learning_completion: vec![0.0; num_states * num_nodes],
            };
            max_nodes.push(MaxNode::Compound(navigate_node));
        }

        let initial_q_value = 0.123; // world.max_reward() / (1 - gamma)

        assert!(max_nodes.len() == start_actions);
        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();

            max_nodes.push(MaxNode::Primitive(PrimitiveNode {
                //PrimitiveNode::new(action, initial_q_value),
                action,
                values: vec![initial_q_value; num_states],
            }));
        }

        assert!(max_nodes.len() == num_nodes);

        MaxQ {
            alpha,
            gamma,
            epsilon,
            show_table,

            state_indexer,

            max_nodes,
            max_node_visits: vec![0; num_nodes],
            start_actions,
        }
    }

    fn evaluate_max_node(
        &self,
        node_index: usize,
        world: &World,
        state: &State,
        state_index: usize,
    ) -> Option<(f64, usize, Actions)> {
        match self.max_nodes[node_index] {
            MaxNode::Primitive(ref primitive) => {
                let value_index = state_index; // primitive.get_index(world, state);
                Some((
                    primitive.values[value_index],
                    node_index,
                    primitive.action,
                ))
            }
            MaxNode::Compound(ref compound) => {
                if !self.terminal_state(compound.compound_type, world, state) {

                    let num_nodes = self.max_nodes.len();

                    let mut highest_q = None;
                    let mut best_index = node_index;
                    let mut best_value = 0.0;
                    let mut best_action = Actions::PickUp;

                    for child_index in &compound.children {
                        if let Some((max_value, _, max_action)) =
                            self.evaluate_max_node(*child_index, world, state, state_index)
                        {

                            let q = Some(
                                max_value +
                                    compound.completion[state_index * num_nodes + child_index],
                            );

                            if q > highest_q {
                                highest_q = q;
                                best_index = *child_index;
                                best_value = max_value;
                                best_action = max_action;
                            }
                        }
                    }

                    if highest_q != None {
                        Some((best_value, best_index, best_action))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn evaluate_max_learning_node(
        &self,
        node_index: usize,
        world: &World,
        state: &State,
        state_index: usize,
    ) -> Option<(f64, usize)> {
        match self.max_nodes[node_index] {
            MaxNode::Primitive(ref primitive) => {
                let value_index = state_index; // primitive.get_index(world, state);
                Some((primitive.values[value_index], node_index))
            }
            MaxNode::Compound(ref compound) => {
                if !self.terminal_state(compound.compound_type, world, state) {

                    let num_nodes = self.max_nodes.len();

                    let mut highest_q = None;
                    let mut best_index = node_index;
                    let mut best_value = 0.0;

                    for child_index in &compound.children {
                        if let Some((max_value, _)) =
                            self.evaluate_max_learning_node(*child_index, world, state, state_index)
                        {

                            let q = Some(
                                max_value +
                                    compound.learning_completion[state_index * num_nodes +
                                                                     child_index],
                            );

                            if q > highest_q {
                                highest_q = q;
                                best_index = *child_index;
                                best_value = max_value;
                            }
                        }
                    }

                    if highest_q != None {
                        Some((best_value, best_index))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn terminal_state(&self, node_type: CompoundNodeType, world: &World, state: &State) -> bool {
        match node_type {
            CompoundNodeType::Root => state.at_destination(),
            CompoundNodeType::Get => state.get_passenger() == None,
            CompoundNodeType::Put => state.get_passenger() != None,
            CompoundNodeType::Navigate(id) => {
                Some(state.get_taxi()) == world.get_fixed_position(id)
            }
        }
    }

    fn learning_reward(&self, _node_index: usize, _world: &World, _state: &State) -> f64 {
        // match self.max_nodes[node_index] {
        //     MaxNode::Compound(ref compound) => {
        //         if !self.terminal_state(compound.compound_type, world, state) {
        //             -100.0
        //         } else {
        //             match compound.compound_type {
        //                 CompoundNodeType::Put => {
        //                     if let Some(id) = state.get_passenger() {
        //                         if id != state.get_destination() {
        //                             -100.0
        //                         } else {
        //                             0.0
        //                         }
        //                     } else {
        //                         0.0
        //                     }
        //                 }
        //                 _ => 0.0,
        //             }
        //         }
        //     }
        //     _ => 0.0,
        // }

        0.0
    }

    fn maxq_q<R: Rng>(
        &mut self,
        node_index: usize,
        world: &World,
        mut state: &mut State,
        max_steps: usize,
        mut rng: &mut R,
    ) -> Vec<usize> {

        let mut seq = Vec::new();

        // Store a copy of the compound node type so that we do not need to hold
        // onto a reference of self.max_nodes[node_index].  This is necessary
        // so that we can recursively call this function and mutate self.
        let compound_node_type = match self.max_nodes[node_index] {
            MaxNode::Primitive(_) => None,
            MaxNode::Compound(ref compound) => Some(compound.compound_type),
        };

        if let Some(node_type) = compound_node_type {
            let num_nodes = self.max_nodes.len();

            while !self.terminal_state(node_type, world, state) && seq.len() < max_steps {

                // println!(
                //     "step {} node {}\n{}",
                //     seq.len(),
                //     node_index,
                //     state.display(world)
                // );

                if let Some(state_index) = self.state_indexer.get_index(world, state) {

                    self.max_node_visits[node_index] += 1;

                    let selected_action: Option<usize> = {

                        let nongreedy_roll = rng.gen_range(0.0f64, 1.0f64);

                        if nongreedy_roll < self.epsilon {
                            let action_offset = rng.gen_range(0, Actions::NUM_ELEMENTS);
                            Some(self.start_actions + action_offset)
                        } else {
                            if let Some((_, child_index, _)) =
                                self.evaluate_max_node(node_index, world, state, state_index)
                            {
                                Some(child_index)
                            } else {
                                None
                            }
                        }
                    };


                    if let Some(child_index) = selected_action {

                        // match self.max_nodes[child_index] {
                        //     MaxNode::Primitive(ref primitive) => {
                        //         println!(
                        //             "Step {} Selecting child {} - primitive action {}",
                        //             seq.len(),
                        //             child_index,
                        //             primitive.action
                        //         )
                        //     }
                        //     MaxNode::Compound(ref compound) => {
                        //         println!(
                        //             "Step {} Selecting child {} - {:?}",
                        //             seq.len(),
                        //             child_index,
                        //             compound.compound_type
                        //         )
                        //     }
                        // };

                        let mut child_seq = self.maxq_q::<R>(
                            child_index,
                            world,
                            &mut state,
                            max_steps - seq.len(),
                            &mut rng,
                        );

                        if child_seq.len() + seq.len() < max_steps {

                            // There should terminal state check should be run for all parents here.
                            // For taxi, there is no way for a parent to terminate
                            // without the current node terminating, so not needed here.
                            // Note that the evaluate_max_node on the result state will test that
                            // the result state is terminal, so the ancestor test is happening but
                            // it is only one layer deep instead of the full stack.

                            if let Some(result_state_index) =
                                self.state_indexer.get_index(world, state)
                            {


                                let learning_reward =
                                    self.learning_reward(node_index, world, state);

                                if let Some((best_next_value, best_next_index)) =
                                    self.evaluate_max_learning_node(
                                        node_index,
                                        world,
                                        state,
                                        result_state_index,
                                    )
                                {

                                    if let MaxNode::Compound(ref mut compound) =
                                        self.max_nodes[node_index]
                                    {

                                        let result_state_value = best_next_value +
                                            compound.completion[result_state_index * num_nodes +
                                                                    best_next_index];
                                        let result_state_learning_value = best_next_value +
                                            compound.learning_completion[result_state_index *
                                                                             num_nodes +
                                                                             best_next_index];
                                        let mut accum_gamma = self.gamma;
                                        for si in child_seq.iter().rev() {

                                            compound.learning_completion[si * num_nodes +
                                                                             child_index] *= 1.0 -
                                                self.alpha;
                                            compound.learning_completion[si * num_nodes +
                                                                             child_index] +=
                                                self.alpha * accum_gamma *
                                                    (learning_reward + result_state_learning_value);

                                            compound.completion[si * num_nodes + child_index] *=
                                                1.0 - self.alpha;
                                            compound.completion[si * num_nodes + child_index] +=
                                                self.alpha * accum_gamma * result_state_value;
                                            accum_gamma *= self.gamma;
                                        }
                                    } else {
                                        panic!("Failed to unwrap compound node {}.", node_index);
                                    }
                                }
                            }

                        }

                        seq.append(&mut child_seq);
                    }
                }
            }

        // match self.max_nodes[node_index] {
        //     MaxNode::Primitive(_) => {}
        //     MaxNode::Compound(ref compound) => {
        //         println!(
        //             "Step {} Terminating node {} - {:?}",
        //             seq.len(),
        //             node_index,
        //             compound.compound_type
        //         );
        //     }
        // };


        } else {
            // Primitive node type
            if let Some(state_index) = self.state_indexer.get_index(world, state) {

                if let MaxNode::Primitive(ref mut primitive) = self.max_nodes[node_index] {

                    let value_index = state_index; // primitive.get_index(world, state);

                    let reward = state.apply_action(world, primitive.action);

                    primitive.values[value_index] *= 1.0 - self.alpha;
                    primitive.values[value_index] += self.alpha * reward;
                    seq.push(state_index);
                } else {
                    panic!("Failed to unwrap primitive node {}.", node_index);
                }
            }
        }

        // assert!(
        //     seq.len() > 0 || max_steps == 0,
        //     "Failed to append to sequence while evaluating node {} - {}\n\
        //     state:\n{}\n\
        //     max_nodes = {:#?}",
        //     node_index,
        //     describe_max_node(&self.max_nodes[node_index]),
        //     state.display(world),
        //     self.max_nodes
        // );
        seq
    }
}


impl Runner for MaxQ {
    fn learn<R: Rng>(
        &mut self,
        world: &World,
        mut state: State,
        max_steps: usize,
        mut rng: &mut R,
    ) -> Option<usize> {

        // println!("Learning:\n{:#?}\n{}\n", state, state.display(world));

        let seq = self.maxq_q(0, world, &mut state, max_steps, &mut rng);

        // println!(
        //     "Finished {} steps:\n{:#?}\n{}\n{}",
        //     seq.len(),
        //     state,
        //     state.display(world),
        //     if state.at_destination() {
        //         "success"
        //     } else {
        //         "failed"
        //     }
        // );

        if state.at_destination() {
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

            if let Some(state_index) = self.state_indexer.get_index(world, &state) {
                if let Some((_, _, next_action)) =
                    self.evaluate_max_node(0, world, &state, state_index)
                {
                    attempt.step(next_action);
                    state.apply_action(world, next_action);
                } else {
                    break;
                }
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

            if let Some(state_index) = self.state_indexer.get_index(world, &state) {
                if let Some((_, _, next_action)) =
                    self.evaluate_max_node(0, world, &state, state_index)
                {
                    state.apply_action(world, next_action);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        state.at_destination()
    }

    fn report_training_result(&self, world: &World) {

        if !self.show_table {
            return;
        }

        let num_nodes = self.max_nodes.len();

        for si in 0..self.state_indexer.num_states() {
            if let Some(state) = self.state_indexer.get_state(world, si) {
                if !state.at_destination() {
                    println!("{}\n{}", si, state.display(world));
                    if let Some((root_value, root_child, action)) =
                        self.evaluate_max_node(0, world, &state, si)
                    {

                        let root_completion =
                            if let MaxNode::Compound(ref compound) = self.max_nodes[root_child] {
                                compound.completion[si * num_nodes + root_child]
                            } else {
                                -123.0
                            };

                        println!(
                            "Result {} => {} + {} = {}",
                            action,
                            root_value,
                            root_completion,
                            root_value + root_completion
                        );

                        let mut current_index = 0;
                        loop {
                            if let Some((value, child_index, _)) =
                                self.evaluate_max_node(current_index, world, &state, si)
                            {
                                match self.max_nodes[current_index] {
                                    MaxNode::Primitive(ref primitive) => {
                                        println!(
                                            "Choose {} {} => {}",
                                            child_index,
                                            primitive.action,
                                            value
                                        );
                                        break;
                                    }
                                    MaxNode::Compound(ref compound) => {
                                        for ci in &compound.children {
                                            if let Some((value, _, _)) =
                                                self.evaluate_max_node(*ci, world, &state, si)
                                            {
                                                let completion =
                                                    compound.completion[si * num_nodes + *ci];
                                                println!(
                                                    "{} {} = {} + {} = {}",
                                                    ci,
                                                    describe_max_node(&self.max_nodes[*ci]),
                                                    value,
                                                    completion,
                                                    value + completion
                                                );

                                            }
                                        }

                                        let completion = compound.completion[si * num_nodes +
                                                                                 child_index];
                                        let q = value + completion;
                                        println!(
                                            "Choose {} {} => {} + {} = {}",
                                            child_index,
                                            describe_max_node(&self.max_nodes[child_index]),
                                            value,
                                            completion,
                                            q
                                        );
                                    }
                                }

                                current_index = child_index;
                            } else {
                                println!("Failed to evaluate");
                                break;
                            }
                        }
                    } else {
                        println!("Failed to evaluate!");
                    }

                    println!("\n");
                }
            }
        }
        // println!("{:#?}", self.max_nodes);
    }
}

fn describe_max_node(max_node: &MaxNode) -> String {
    match *max_node {
        MaxNode::Primitive(ref primitive) => format!("{:?}", primitive.action),
        MaxNode::Compound(ref compound) => format!("{:?}", compound.compound_type),
    }
}
