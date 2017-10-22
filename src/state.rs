

use rand::Rng;

use position::Position;
use world::World;
use actions::Actions;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct State {
    taxi: Position,
    passenger: Option<char>,
    destination: char,
}

impl State {
    pub fn build(
        world: &World,
        taxi_pos: (i32, i32),
        passenger_id: char,
        destination_id: char,
    ) -> Result<State, String> {

        if taxi_pos.0 < 0 || taxi_pos.0 >= world.width || taxi_pos.1 < 0 ||
            taxi_pos.1 >= world.height
        {
            Err(format!(
                "Taxi position ({},{}) is invalid, world (width, height) is ({},{}).",
                taxi_pos.0,
                taxi_pos.1,
                world.width,
                world.height
            ))
        } else {

            match world.get_fixed_position(destination_id) {
                None => Err(format!(
                    "Failed to find destination location '{}'",
                    destination_id
                )),
                Some(_) => {
                    match world.get_fixed_position(passenger_id) {
                        None => Err(format!(
                            "Failed to find passenger location '{}'",
                            passenger_id
                        )),
                        Some(passenger_pos) => {
                            let taxi = Position::new(taxi_pos.0, taxi_pos.1);
                            let passenger =
                                if *passenger_pos != taxi || passenger_id == destination_id {
                                    Some(passenger_id)
                                } else {
                                    None
                                };
                            let destination = destination_id;

                            Ok(State {
                                taxi,
                                passenger,
                                destination,
                            })
                        }
                    }
                }
            }
        }
    }

    pub fn build_random<R: Rng>(world: &World, rng: &mut R) -> Result<State, String> {

        let taxi_x = rng.gen_range(0, world.width);
        let taxi_y = rng.gen_range(0, world.height);

        let num_fixed_positions = world.fixed_positions.len();

        if num_fixed_positions < 2 {
            return Err(format!(
                "World does not have enough fixed positions. Need at least 2, but only have {}.",
                num_fixed_positions
            ));
        }

        let passenger_fp_index = rng.gen_range(0, num_fixed_positions);
        let destination_fp_index = (passenger_fp_index + rng.gen_range(1, num_fixed_positions)) %
            num_fixed_positions;

        let passenger = Some(world.fixed_positions[passenger_fp_index].id);

        let destination = world.fixed_positions[destination_fp_index].id;

        Ok(State {
            taxi: Position::new(taxi_x, taxi_y),
            passenger,
            destination,
        })
    }

    pub fn display(&self, world: &World) -> String {

        let world_strings = world.display_strings();

        let mut result = String::new();

        let mut current_position = Position::new(0, 0);

        for (i_r, r) in world_strings.iter().enumerate() {

            if i_r % 2 == 1 {

                for (i_c, c) in r.chars().enumerate() {

                    if i_c % 2 == 1 {

                        result.push(self.calc_character(c, &current_position));

                        current_position.x += 1;
                    } else {
                        result.push(c);
                    }
                }

                current_position.x = 0;
                current_position.y += 1;

            } else {
                result += r;
            }

            result.push('\n');
        }

        result
    }

    fn calc_character(&self, id: char, position: &Position) -> char {

        if id == self.destination {
            match self.passenger {
                Some(passenger_id) if passenger_id == self.destination => 'D',
                _ => 'd',
            }
        } else {

            match self.passenger {

                Some(passenger_id) => {
                    if passenger_id == id {
                        'p'
                    } else if self.taxi == *position {
                        't'
                    } else {
                        '.'
                    }
                }
                None => if self.taxi == *position { 'T' } else { '.' },
            }

        }
    }

    pub fn apply_action(&self, world: &World, action: Actions) -> State {

        if !world.valid_action(&self.taxi, action) {
            *self
        } else {

            let delta = position_delta(action);

            let new_taxi = self.taxi + delta;

            let new_passenger = match self.passenger {
                Some(passenger_id) => {
                    match world.get_fixed_position(passenger_id) {
                        Some(passenger_pos)
                            if *passenger_pos == new_taxi && passenger_id != self.destination => {
                            None
                        }
                        _ => self.passenger,
                    }
                }

                None => {
                    // passenger is in taxi, should they get out?
                    match world.get_fixed_position(self.destination) {
                        Some(destination_pos) if *destination_pos == new_taxi => Some(
                            self.destination,
                        ),

                        _ => self.passenger,
                    }
                }
            };

            State {
                taxi: new_taxi,
                passenger: new_passenger,
                ..*self
            }
        }
    }

    pub fn at_destination(&self) -> bool {
        if let Some(passenger_id) = self.passenger {
            passenger_id == self.destination
        } else {
            false
        }
    }
}


fn position_delta(action: Actions) -> Position {
    match action {
        Actions::North => Position::new(0, -1),
        Actions::South => Position::new(0, 1),
        Actions::East => Position::new(1, 0),
        Actions::West => Position::new(-1, 0),
    }
}


#[cfg(test)]
mod test_state {

    use rand::thread_rng;
    use super::*;

    #[test]
    fn build_correct() {
        let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│G . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│Y .│B .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source_world) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let expected_state = State {
                    taxi: Position::new(1, 3),
                    passenger: Some('R'),
                    destination: 'B',
                };

                match State::build(&w, (1, 3), 'R', 'B') {
                    Err(msg) => panic!(msg),
                    Ok(res_state) => assert_eq!(res_state, expected_state),
                }

            }
        }
    }

    #[test]
    fn build_fails_unknown_passenger() {
        let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│G . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│Y .│B .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source_world) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build(&w, (1, 3), 'C', 'B') {
                    Err(_) => (), // panic!(msg),
                    Ok(res_state) => panic!("Found valid passenger: {:?}", res_state.passenger),
                }
            }
        }
    }

    #[test]
    fn build_fails_unknown_destination() {
        let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│G . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│Y .│B .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source_world) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build(&w, (1, 3), 'Y', 'Q') {
                    Err(_) => (), // panic!(msg),
                    Ok(res_state) => panic!("Found valid destination: {:?}", res_state.destination),
                }
            }
        }
    }

    #[test]
    fn build_fails_invalid_taxi() {
        let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│G . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│Y .│B .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source_world) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build(&w, (1, 6), 'R', 'B') {
                    Err(_msg) => (), //panic!(_msg),
                    Ok(res_state) => panic!("Found valid taxi: {:?}", res_state.taxi),
                }
            }
        }
    }

    #[test]
    fn passenger_moves_in_taxi() {
        let source = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│R .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        let expected = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. T . . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build(&w, (1, 3), 'R', 'G') {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        assert_eq!(state.passenger, None);

                        let result = state.apply_action(&w, Actions::North);
                        assert_eq!(result.passenger, None);

                        let result_str = result.display(&w);
                        assert_eq!(expected, result_str);
                    }
                }
            }
        }
    }

    #[test]
    fn passenger_stays_at_destination() {
        let source = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│R .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        let expected0 = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│D .│. .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        let expected1 = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│D t│. .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build(&w, (1, 4), 'R', 'R') {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        assert_eq!(state.passenger, Some('R'));
                        assert!(state.at_destination());

                        let result0 = state.apply_action(&w, Actions::North);
                        assert_eq!(result0.passenger, Some('R'));
                        assert!(result0.at_destination());

                        let result0_str = result0.display(&w);
                        assert_eq!(expected0, result0_str);

                        let result1 = result0.apply_action(&w, Actions::East);
                        assert_eq!(result1.passenger, Some('R'));
                        assert!(result1.at_destination());

                        let result1_str = result1.display(&w);
                        assert_eq!(expected1, result1_str);
                    }
                }
            }
        }
    }

    #[test]
    fn passenger_picked_up_in_taxi() {
        let source = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. R . . .│\n\
        │         │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        let expected0 = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. T . . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        let expected1 = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │T . . . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build(&w, (1, 3), 'R', 'G') {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let result0 = state.apply_action(&w, Actions::North);
                        let result0_str = result0.display(&w);
                        assert_eq!(result0.passenger, None);
                        assert_eq!(expected0, result0_str);

                        let result1 = result0.apply_action(&w, Actions::West);
                        let result1_str = result1.display(&w);
                        assert_eq!(result1.passenger, None);
                        assert_eq!(expected1, result1_str);
                    }
                }
            }
        }
    }

    #[test]
    fn passenger_dropped_destination() {
        let source = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│R .│. .│\n\
        │ │   │   │\n\
        │.│G .│. .│\n\
        └─┴───┴───┘\n\
        ";

        let expected = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│. .│\n\
        │ │   │   │\n\
        │.│D .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build(&w, (1, 3), 'R', 'G') {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        assert_eq!(state.passenger, None);

                        let result = state.apply_action(&w, Actions::South);
                        assert_eq!(result.passenger, Some('G'));
                        assert!(result.at_destination());

                        let result_str = result.display(&w);
                        assert_eq!(expected, result_str);
                    }
                }
            }
        }
    }

    #[test]
    fn passenger_not_dropped_other_fixed_position() {
        let source = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│R Y│. .│\n\
        │ │   │   │\n\
        │.│G .│. .│\n\
        └─┴───┴───┘\n\
        ";

        let expected = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. T│. .│\n\
        │ │   │   │\n\
        │.│d .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build(&w, (1, 3), 'R', 'G') {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        assert_eq!(state.passenger, None);

                        let result = state.apply_action(&w, Actions::East);
                        assert_eq!(result.passenger, None);
                        assert!(!result.at_destination());

                        let result_str = result.display(&w);
                        assert_eq!(expected, result_str);
                    }
                }
            }
        }
    }

    #[test]
    fn passenger_picked_up_and_dropped_off() {
        let source = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. R . . .│\n\
        │         │\n\
        │.│G .│. .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        let expected0 = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. T . . .│\n\
        │         │\n\
        │.│d .│. .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        let expected1 = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│D .│. .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build(&w, (2, 2), 'R', 'G') {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        assert_eq!(state.passenger, Some('R'));

                        let result0 = state.apply_action(&w, Actions::West);
                        assert_eq!(result0.passenger, None);
                        let result0_str = result0.display(&w);
                        assert_eq!(expected0, result0_str);

                        let result1 = result0.apply_action(&w, Actions::South);
                        assert_eq!(result1.passenger, Some('G'));

                        let result1_str = result1.display(&w);
                        assert_eq!(expected1, result1_str);
                    }
                }
            }
        }
    }

    #[test]
    fn build_random_state() {
        let source_world = "\
    ┌───┬─────┐\n\
    │R .│. . G│\n\
    │   │     │\n\
    │. .│. . .│\n\
    │         │\n\
    │. . . . .│\n\
    │         │\n\
    │.│. .│. .│\n\
    │ │   │   │\n\
    │Y│. .│B .│\n\
    └─┴───┴───┘\n\
    ";

        let w = World::build_from_str(source_world).unwrap();

        let mut rng = thread_rng();

        for _ in 0..20 {

            let state = State::build_random(&w, &mut rng).unwrap();

            println!("{:?}", state);
            assert!(state.taxi.x >= 0);
            assert!(state.taxi.x < w.width);
            assert!(state.taxi.y >= 0);
            assert!(state.taxi.y < w.height);

            let passenger_id = state.passenger.unwrap();

            let passenger_fp_position = w.fixed_positions.iter().position(
                |fp| fp.id == passenger_id,
            );

            assert_ne!(passenger_fp_position, None);

            let destination_fp_position = w.fixed_positions.iter().position(
                |fp| fp.id == state.destination,
            );

            assert_ne!(destination_fp_position, None);

            assert_ne!(passenger_fp_position, destination_fp_position);
        }
    }
}
