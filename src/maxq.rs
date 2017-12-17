use std;

use rand::Rng;

use state::State;
use actions::Actions;
use world::World;
use state_indexer::StateIndexer;

use runner::{Runner, Attempt};


#[derive(Debug, Clone)]
struct PrimitiveNode {
    action: Actions,
    values: Vec<f64>,
}

impl PrimitiveNode {
    fn new(action: Actions, initial_q_value: f64, world: &World) -> PrimitiveNode {
        let num_values = match action {
            Actions::PickUp | Actions::DropOff => 2,
            Actions::North | Actions::South | Actions::East | Actions::West => 1,
        };

        PrimitiveNode {
            action,
            values: vec![initial_q_value; num_values],
        }
    }

    fn get_index(&self, world: &World, state: &State) -> usize {
        match self.action {
            // Pick-up has only 2 results, taxi is at passenger or not.
            Actions::PickUp => {
                match state.get_passenger() {
                    Some(id) if world.get_fixed_position(id) == Some(state.get_taxi()) => 0,
                    _ => 1,
                }
            }

            // Drop-off has only 2 results, passenger is in taxi and at destination or not.
            Actions::DropOff => {
                match world.get_fixed_id(state.get_taxi()) {
                    Some(id) if state.get_passenger() == None && id == state.get_destination() => 0,
                    _ => 1,
                }
            }

            // reward for directional movement is independent of taxi position
            Actions::North | Actions::South | Actions::East | Actions::West => 0,
        }
    }
}


#[derive(Debug, Clone, Copy)]
enum CompoundNodeType {
    Root,
    Get,
    Put,
    Navigate(char),
}

impl CompoundNodeType {
    fn terminal_state(&self, world: &World, state: &State) -> bool {
        match *self {
            CompoundNodeType::Root => state.at_destination(),
            CompoundNodeType::Get => state.get_passenger() == None,
            CompoundNodeType::Put => state.get_passenger() != None,
            CompoundNodeType::Navigate(id) => {
                Some(state.get_taxi()) == world.get_fixed_position(id)
            }
        }
    }

    fn value_count(&self, world: &World) -> usize {
        match *self {
            CompoundNodeType::Root => 1,
            CompoundNodeType::Get => {
                let qpickup_count = (world.width as usize) * (world.height as usize) *
                    world.num_fixed_positions();
                let qnavigate_count = world.num_fixed_positions();
                qpickup_count + qnavigate_count
            }

            CompoundNodeType::Put => 1,
            CompoundNodeType::Navigate(_) => world.num_fixed_positions(),
        }
    }

    fn get_index(
        &self,
        child_type: MaxNodeType,
        world: &World,
        state: &State,
        state_indexer: &StateIndexer,
    ) -> Option<usize> {
        match *self {
            CompoundNodeType::Root => {
                match child_type {
                    MaxNodeType::Compound(compound_type) => {
                        match compound_type {
                            CompoundNodeType::Get => {
                                if let Some(passenger_id) = state.get_passenger() {
                                    if let Some(passenger_index) =
                                        world.get_fixed_index(passenger_id)
                                    {
                                        if let Some(destination_index) =
                                            world.get_fixed_index(state.get_destination())
                                        {
                                            return Some(
                                                passenger_index * world.num_fixed_positions() +
                                                    destination_index,
                                            );
                                        }
                                    }
                                }

                                None
                            }

                            CompoundNodeType::Put => None,

                            _ => None,
                        }
                    }

                    MaxNodeType::Primitive(_) => state_indexer.get_index(world, state),
                }
            }

            CompoundNodeType::Get => {
                match child_type {
                    MaxNodeType::Primitive(action) => {
                        match action {
                            Actions::PickUp => {
                                if let Some(passenger_id) = state.get_passenger() {
                                    if let Some(passenger_index) =
                                        world.get_fixed_index(passenger_id)
                                    {
                                        let mut result = passenger_index;

                                        result *= world.height as usize;
                                        result += state.get_taxi().y as usize;

                                        result *= world.width as usize;
                                        result += state.get_taxi().x as usize;

                                        return Some(result);
                                    }
                                }

                                None
                            }

                            _ => state_indexer.get_index(world, state),
                        }
                    }

                    MaxNodeType::Compound(compound_type) => {
                        match compound_type {
                            CompoundNodeType::Navigate(_) => {
                                if let Some(passenger_id) = state.get_passenger() {
                                    if let Some(passenger_index) =
                                        world.get_fixed_index(passenger_id)
                                    {
                                        Some(passenger_index)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }

                            _ => None,
                        }
                    }
                }
            }
            CompoundNodeType::Put => {
                match child_type {
                    MaxNodeType::Primitive(action) => {
                        match action {
                            Actions::DropOff => {
                                if let Some(destination_index) =
                                    world.get_fixed_index(state.get_destination())
                                {
                                    let mut result = destination_index;

                                    result *= world.height as usize;
                                    result += state.get_taxi().y as usize;

                                    result *= world.width as usize;
                                    result += state.get_taxi().x as usize;

                                    return Some(result);
                                }

                                None
                            }

                            _ => state_indexer.get_index(world, state),
                        }
                    }

                    MaxNodeType::Compound(compound_type) => {
                        match compound_type {
                            CompoundNodeType::Navigate(_) => {
                                if let Some(destination_index) =
                                    world.get_fixed_index(state.get_destination())
                                {
                                    Some(destination_index)
                                } else {
                                    None
                                }
                            }

                            _ => None,
                        }
                    }
                }
            }
            CompoundNodeType::Navigate(id) => {
                match child_type {
                    MaxNodeType::Compound(_) => None,
                    MaxNodeType::Primitive(action) => {
                        match action {
                            Actions::North | Actions::South | Actions::East | Actions::West => {
                                if let Some(id_index) = world.get_fixed_index(id) {
                                    let mut result = id_index;

                                    result *= world.height as usize;
                                    result += state.get_taxi().y as usize;


                                    result *= world.width as usize;
                                    result += state.get_taxi().x as usize;

                                    Some(result)
                                } else {
                                    None
                                }
                            }

                            _ => None,
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct CompoundNode {
    compound_type: CompoundNodeType,
    children: Vec<usize>,
    completion: Vec<f64>, // indexed by value_index*num_nodes + node_index
    learning_completion: Vec<f64>,
}

impl CompoundNode {
    fn new(
        compound_type: CompoundNodeType,
        children: Vec<usize>,
        num_nodes: usize,
        world: &World,
        num_states: usize,
    ) -> CompoundNode {

        //let num_values = compound_type.value_count(world);
        let completion_size = num_states * num_nodes;

        CompoundNode {
            compound_type,
            children,
            completion: vec![0.0; completion_size],
            learning_completion: vec![0.0; completion_size],
        }
    }

    // Convienence function
    fn terminal_state(&self, world: &World, state: &State) -> bool {
        self.compound_type.terminal_state(world, state)
    }

    // Convienence function
    fn get_index(
        &self,
        child_type: MaxNodeType,
        world: &World,
        state: &State,
        state_indexer: &StateIndexer,
    ) -> Option<usize> {
        self.compound_type.get_index(
            child_type,
            world,
            state,
            state_indexer,
        )
    }
}

#[derive(Debug, Clone, Copy)]
enum MaxNodeType {
    Primitive(Actions),
    Compound(CompoundNodeType),
}

#[derive(Debug, Clone)]
enum MaxNode {
    Primitive(PrimitiveNode),
    Compound(CompoundNode),
}

impl MaxNode {
    fn get_type(&self) -> MaxNodeType {
        match *self {
            MaxNode::Primitive(ref primitive) => MaxNodeType::Primitive(primitive.action),
            MaxNode::Compound(ref compound) => MaxNodeType::Compound(compound.compound_type),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaxQ {
    alpha: f64,
    gamma: f64,
    epsilon: f64,
    show_table: bool,

    max_nodes: Vec<MaxNode>,

    state_indexer: StateIndexer,

    start_actions: usize,
}

impl MaxQ {
    pub fn new(world: &World, alpha: f64, gamma: f64, epsilon: f64, show_table: bool) -> MaxQ {

        let state_indexer = StateIndexer::new(world);
        let num_states = state_indexer.num_states();

        let num_destinations = world.num_fixed_positions();
        let num_navigates = num_destinations;

        let start_get = 1;
        let start_put = start_get + 1;
        let start_navigates = start_put + 1;
        let start_actions = start_navigates + num_navigates;

        let num_nodes = start_actions + Actions::NUM_ELEMENTS;

        let mut max_nodes = Vec::with_capacity(num_nodes);


        let root_node = CompoundNode::new(
            CompoundNodeType::Root,
            vec![start_get, start_put],
            num_nodes,
            world,
            num_states,
        );
        max_nodes.push(MaxNode::Compound(root_node));

        assert!(max_nodes.len() == start_get);
        let get_node = {

            let num_children = 1 + num_destinations;
            let mut children = Vec::with_capacity(num_children);

            for i in 0..num_destinations {
                children.push(start_navigates + i);
            }

            children.push(start_actions + Actions::PickUp.to_index());

            CompoundNode::new(
                CompoundNodeType::Get,
                children,
                num_nodes,
                world,
                num_states,
            )
        };
        max_nodes.push(MaxNode::Compound(get_node));


        assert!(max_nodes.len() == start_put);
        let put_node = {
            let num_children = 1 + num_destinations;
            let mut children = Vec::with_capacity(num_children);
            children.push(start_actions + Actions::DropOff.to_index());

            for i in 0..num_destinations {
                children.push(start_navigates + i);
            }

            CompoundNode::new(
                CompoundNodeType::Put,
                children,
                num_nodes,
                world,
                num_states,
            )
        };
        max_nodes.push(MaxNode::Compound(put_node));

        assert!(max_nodes.len() == start_navigates);

        for fixed_position_index in 0..num_destinations {
            let id = world.get_fixed_id_from_index(fixed_position_index).unwrap();

            let children = vec![
                start_actions + Actions::North.to_index(),
                start_actions + Actions::South.to_index(),
                start_actions + Actions::East.to_index(),
                start_actions + Actions::West.to_index(),
            ];

            let navigate_node = CompoundNode::new(
                CompoundNodeType::Navigate(id),
                children,
                num_nodes,
                world,
                num_states,
            );
            max_nodes.push(MaxNode::Compound(navigate_node));
        }

        let initial_q_value = 0.123; // world.max_reward() / (1 - gamma)

        assert!(max_nodes.len() == start_actions);
        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();

            max_nodes.push(MaxNode::Primitive(
                PrimitiveNode::new(action, initial_q_value, world),
            ));
        }

        assert!(max_nodes.len() == num_nodes);

        MaxQ {
            alpha,
            gamma,
            epsilon,
            show_table,

            max_nodes,

            state_indexer,

            start_actions,
        }
    }

    fn evaluate_max_node(
        &self,
        node_index: usize,
        world: &World,
        state: &State,
    ) -> Option<(f64, usize, Actions)> {
        match self.max_nodes[node_index] {
            MaxNode::Primitive(ref primitive) => {
                let value_index = primitive.get_index(world, state);
                Some((
                    primitive.values[value_index],
                    node_index,
                    primitive.action,
                ))
            }
            MaxNode::Compound(ref compound) => {
                if !compound.terminal_state(world, state) {

                    let num_nodes = self.max_nodes.len();

                    let mut highest_q = None;
                    let mut best_index = node_index;
                    let mut best_value = 0.0;
                    let mut best_action = Actions::PickUp;

                    for child_index in &compound.children {
                        if let Some((max_value, _, max_action)) =
                            self.evaluate_max_node(*child_index, world, state)
                        {
                            let completion = if let Some(value_index) =
                                compound.get_index(
                                    self.max_nodes[*child_index].get_type(),
                                    world,
                                    state,
                                    &self.state_indexer,
                                )
                            {
                                compound.completion[value_index * num_nodes + child_index]
                            } else {
                                0.0
                            };

                            let q = Some(max_value + completion);

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
    ) -> Option<(f64, usize)> {
        match self.max_nodes[node_index] {
            MaxNode::Primitive(ref primitive) => {
                let value_index = primitive.get_index(world, state);
                Some((primitive.values[value_index], node_index))
            }
            MaxNode::Compound(ref compound) => {
                if !compound.terminal_state(world, state) {

                    let num_nodes = self.max_nodes.len();

                    let mut highest_q = None;
                    let mut best_index = node_index;
                    let mut best_value = 0.0;

                    for child_index in &compound.children {
                        if let Some((max_value, _)) =
                            self.evaluate_max_learning_node(*child_index, world, state)
                        {
                            let learning_completion = if let Some(value_index) =
                                compound.get_index(
                                    self.max_nodes[*child_index].get_type(),
                                    world,
                                    state,
                                    &self.state_indexer,
                                )
                            {
                                compound.learning_completion[value_index * num_nodes + child_index]
                            } else {
                                0.0
                            };

                            let q = Some(max_value + learning_completion);

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

    fn learning_reward(&self, _node_index: usize, _world: &World, _state: &State) -> f64 {
        0.0
        // match self.max_nodes[node_index] {
        //     MaxNode::Compound(ref compound) => {
        //         if !self.terminal_state(compound.compound_type, world, state) {
        //             0.0
        //         } else {
        //             match compound.compound_type {
        //                 CompoundNodeType::Put => if state.at_destination() {
        //                  0.0
        //                  } else { -100.0 },
        //                 _ => 0.0,
        //             }
        //         }
        //     }

        //     _ => 0.0,
        // }
    }

    fn maxq_q<R: Rng>(
        &mut self,
        node_index: usize,
        world: &World,
        mut state: State,
        max_steps: usize,
        mut rng: &mut R,
    ) -> (State, Vec<State>) {

        // Store a copy of the compound node type so that we do not need to hold
        // onto a reference of self.max_nodes[node_index].  This is necessary
        // so that we can recursively call this function and mutate self.
        let compound_node_type = match self.max_nodes[node_index] {
            MaxNode::Primitive(_) => None,
            MaxNode::Compound(ref compound) => Some(compound.compound_type),
        };

        if let Some(node_type) = compound_node_type {
            let num_nodes = self.max_nodes.len();

            let mut seq = Vec::new();

            while !node_type.terminal_state(world, &state) && seq.len() < max_steps {

                // println!(
                //     "step {} node {}\n{}",
                //     seq.len(),
                //     node_index,
                //     state.display(world)
                // );

                let selected_action: Option<usize> = {

                    let nongreedy_roll = rng.gen_range(0.0f64, 1.0f64);

                    if nongreedy_roll < self.epsilon {
                        let action_offset = rng.gen_range(0, Actions::NUM_ELEMENTS);
                        Some(self.start_actions + action_offset)
                    } else if let Some((_, child_index, _)) =
                        self.evaluate_max_node(node_index, world, &state)
                    {
                        Some(child_index)
                    } else {
                        None
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

                    let child_type = self.max_nodes[child_index].get_type();

                    let (next_state, mut child_seq) = self.maxq_q::<R>(
                        child_index,
                        world,
                        state,
                        max_steps - seq.len(),
                        &mut rng,
                    );

                    // A terminal state check should be run for all parents here.
                    // For taxi, there is no way for a parent to terminate
                    // without the current node terminating, so not needed here.

                    let finished_successfully = match child_type {
                        MaxNodeType::Primitive(_) => true,
                        MaxNodeType::Compound(ref child_compound) => {
                            child_compound.terminal_state(world, &next_state)
                        }
                    };

                    if finished_successfully {

                        let learning_reward = self.learning_reward(node_index, world, &next_state);

                        let max_learning_node =
                            self.evaluate_max_learning_node(node_index, world, &next_state);

                        let next_value_index = node_type.get_index(
                            child_type,
                            world,
                            &next_state,
                            &self.state_indexer,
                        );

                        // println(
                        //     "maxq_q {} - {} for state:\n{}",
                        //     node_index,
                        //     describe_max_node(self.max_nodes[node_index]),
                        //     next_state.display(world),
                        // );


                        if let MaxNode::Compound(ref mut compound) = self.max_nodes[node_index] {

                            let (result_state_value, result_state_learning_value) =
                                if let Some(next_value_index) = next_value_index {
                                    if let Some((best_next_value, best_next_index)) =
                                        max_learning_node
                                    {
                                        (
                                            best_next_value +
                                                compound.completion[next_value_index * num_nodes +
                                                                        best_next_index],
                                            best_next_value +
                                                compound.learning_completion[next_value_index *
                                                                                 num_nodes +
                                                                                 best_next_index],
                                        )
                                    } else {
                                        (0.0, 0.0)
                                    }
                                } else {
                                    (0.0, 0.0)
                                };

                            let mut accum_gamma = self.gamma;
                            for child_state in child_seq.iter().rev() {

                                if let Some(value_index) =
                                    compound.get_index(
                                        child_type,
                                        world,
                                        &child_state,
                                        &self.state_indexer,
                                    )
                                {
                                    compound.learning_completion[value_index * num_nodes +
                                                                     child_index] *= 1.0 -
                                        self.alpha;
                                    compound.learning_completion[value_index * num_nodes +
                                                                     child_index] += self.alpha *
                                        accum_gamma *
                                        (learning_reward + result_state_learning_value);


                                    compound.completion[value_index * num_nodes + child_index] *=
                                        1.0 - self.alpha;
                                    compound.completion[value_index * num_nodes + child_index] +=
                                        self.alpha * accum_gamma * result_state_value;

                                    accum_gamma *= self.gamma;
                                }
                            }
                        } else {
                            panic!("Failed to unwrap compound node {}.", node_index);
                        }
                    }


                    seq.append(&mut child_seq);
                    state = next_state;
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

            (state, seq)

        } else {
            // Primitive node type

            if let MaxNode::Primitive(ref mut primitive) = self.max_nodes[node_index] {

                let value_index = primitive.get_index(world, &state);

                let (reward, next_state) = state.apply_action(world, primitive.action);

                primitive.values[value_index] *= 1.0 - self.alpha;
                primitive.values[value_index] += self.alpha * reward;

                (next_state, vec![state])
            } else {
                panic!("Failed to unwrap primitive node {}.", node_index);
            }
        }
    }
}


impl Runner for MaxQ {
    fn learn<R: Rng>(
        &mut self,
        world: &World,
        state: State,
        max_steps: usize,
        mut rng: &mut R,
    ) -> Option<usize> {

        // println!("Learning:\n{:#?}\n{}\n", state, state.display(world));

        let (final_state, seq) = self.maxq_q(0, world, state, max_steps, &mut rng);

        // println!(
        //     "Finished {} steps:\n{:#?}\n{}\n{}",
        //     seq.len(),
        //     final_state,
        //     final_state.display(world),
        //     if final_state.at_destination() {
        //         "success"
        //     } else {
        //         "failed"
        //     }
        // );

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

            if let Some((_, _, next_action)) = self.evaluate_max_node(0, world, &state) {
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

            if let Some((_, _, next_action)) = self.evaluate_max_node(0, world, &state) {
                let (_, next_state) = state.apply_action(world, next_action);
                state = next_state;
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

        let state_indexer = StateIndexer::new(world);

        for si in 0..state_indexer.num_states() {
            if let Some(state) = state_indexer.get_state(world, si) {
                if !state.at_destination() {
                    println!("{}\n{}", si, state.display(world));
                    if let Some((_, _, action)) = self.evaluate_max_node(0, world, &state) {
                        println!(
                            "Result {}",
                            action,
                        );

                        let mut current_index = 0;
                        loop {
                            if let Some((value, child_index, _)) =
                                self.evaluate_max_node(current_index, world, &state)
                            {
                                match self.max_nodes[current_index] {
                                    MaxNode::Primitive(ref primitive) => {
                                        println!(
                                            "Choose {} {} ({}) => {}",
                                            child_index,
                                            primitive.action,
                                            value,
                                            primitive.get_index(world, &state),
                                        );
                                        break;
                                    }
                                    MaxNode::Compound(ref compound) => {
                                        let completion = if let Some(value_index) =
                                            compound.get_index(
                                                self.max_nodes[child_index].get_type(),
                                                world,
                                                &state,
                                                &self.state_indexer,
                                            )
                                        {
                                            compound.completion[value_index * num_nodes +
                                                                    child_index]
                                        } else {
                                            0.0
                                        };

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


                    for (node_index, max_node) in self.max_nodes.iter().enumerate() {
                        match *max_node {
                            MaxNode::Primitive(ref primitive) => {

                                let value_index = primitive.get_index(world, &state);
                                println!(
                                    "{} {}: {} => {}",
                                    node_index,
                                    primitive.action,
                                    value_index,
                                    primitive.values[value_index]
                                );
                            }
                            MaxNode::Compound(ref compound) => {
                                if !compound.terminal_state(world, &state) {
                                    println!("{} {}:", node_index, describe_max_node(&max_node));

                                    let mut visited_nodes = std::collections::BTreeSet::new();

                                    for ci in &compound.children {
                                        visited_nodes.insert(*ci);

                                        let child_type = self.max_nodes[*ci].get_type();

                                        if let Some((value, _, _)) =
                                            self.evaluate_max_node(*ci, world, &state)
                                        {
                                            if let Some(value_index) =
                                                compound.get_index(
                                                    child_type,
                                                    world,
                                                    &state,
                                                    &self.state_indexer,
                                                )
                                            {
                                                let completion_index = value_index * num_nodes +
                                                    *ci;
                                                let completion = compound.completion
                                                    [completion_index];
                                                println!(
                                                    "  {} ({},{}) -> {} => {} + {} = {}",
                                                    describe_max_node(&self.max_nodes[*ci]),
                                                    value_index,
                                                    ci,
                                                    completion_index,
                                                    value,
                                                    completion,
                                                    value + completion
                                                );
                                            } else {
                                                println!(
                                                    "  {} ({}) => {}",
                                                    describe_max_node(&self.max_nodes[*ci]),
                                                    ci,
                                                    value
                                                );

                                            }
                                        }
                                    }

                                    println!("  ----");

                                    for node_index in self.start_actions..self.max_nodes.len() {
                                        if !visited_nodes.contains(&node_index) {
                                            let child_type = self.max_nodes[node_index].get_type();

                                            if let Some((value, _, _)) =
                                                self.evaluate_max_node(node_index, world, &state)
                                            {
                                                if let Some(value_index) =
                                                    compound.get_index(
                                                        child_type,
                                                        world,
                                                        &state,
                                                        &self.state_indexer,
                                                    )
                                                {
                                                    let completion_index = value_index * num_nodes +
                                                        node_index;
                                                    let completion = compound.completion
                                                        [completion_index];
                                                    println!(
                                                        "  {} ({},{}) -> {} => {} + {} = {}",
                                                        describe_max_node(&self.max_nodes[node_index]),
                                                        value_index,
                                                        node_index,
                                                        completion_index,
                                                        value,
                                                        completion,
                                                        value + completion
                                                    );
                                                } else {
                                                    println!(
                                                        "  {} ({}) => {}",
                                                        describe_max_node(&self.max_nodes[node_index]),
                                                        node_index,
                                                        value
                                                    );

                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    println!("======\n");
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
