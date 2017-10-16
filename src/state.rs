

use position::Position;
use world::World;
use actions::Actions;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Taxi {
    position: Position,
}

impl Taxi {
    fn new(x: i32, y: i32) -> Taxi {
        Taxi { position: Position::new(x, y) }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Passenger {
    position: Position,
}

impl Passenger {
    fn new(x: i32, y: i32) -> Passenger {
        Passenger { position: Position::new(x, y) }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Destination {
    position: Position,
}

impl Destination {
    fn new(x: i32, y: i32) -> Destination {
        Destination { position: Position::new(x, y) }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct State {
    taxi: Taxi,
    passenger: Passenger,
    destination: Destination,
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

            match world.get_fixed_position(passenger_id) {
                None => Err(format!(
                    "Failed to find passenger location '{}'",
                    passenger_id
                )),
                Some(passenger_pos) => {
                    match world.get_fixed_position(destination_id) {
                        None => Err(format!(
                            "Failed to find destination location '{}'",
                            destination_id
                        )),
                        Some(destination_pos) => {
                            Ok(State {
                                taxi: Taxi::new(taxi_pos.0, taxi_pos.1),
                                passenger: Passenger::new(passenger_pos.x, passenger_pos.y),
                                destination: Destination::new(destination_pos.x, destination_pos.y),
                            })
                        }
                    }
                }
            }
        }
    }

    pub fn display(&self, world: &World) -> String {

        let world_strings = world.display_strings();

        let mut result = String::new();

        let mut current_position = Position::new(0, 0);

        for (i_r, r) in world_strings.iter().enumerate() {

            if i_r % 2 == 1 {

                for (i_c, c) in r.chars().enumerate() {

                    if i_c % 2 == 1 {

                        result.push(self.calc_character(&current_position));

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

    fn calc_character(&self, position: &Position) -> char {

        if self.taxi.position == *position {
            if self.passenger.position == *position {
                if self.destination.position == *position {
                    'D'
                } else {
                    'T'
                }
            } else {
                't'
            }
        } else if self.passenger.position == *position {
            if self.destination.position == *position {
                'D'
            } else {
                'p'
            }
        } else if self.destination.position == *position {
            'd'
        } else {
            '.'
        }

    }

    pub fn apply_action(&self, world: &World, action: Actions) -> State {

        if !world.valid_action(&self.taxi.position, action) {
            *self
        } else {

            let delta = position_delta(action);

            let new_taxi_pos = self.taxi.position + delta;

            let new_taxi = Taxi {
                position: new_taxi_pos,
                ..self.taxi
            };

            let new_passenger = if self.taxi.position == self.passenger.position {
                Passenger {
                    position: new_taxi_pos,
                    ..self.passenger
                }
            } else {
                self.passenger
            };

            State {
                taxi: new_taxi,
                passenger: new_passenger,
                ..*self
            }
        }
    }

    pub fn at_destination(&self) -> bool {
        self.passenger.position == self.destination.position
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
                    taxi: Taxi::new(1, 3),
                    passenger: Passenger::new(0, 0),
                    destination: Destination::new(3, 3),
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
                        let result = state.apply_action(&w, Actions::North);
                        let result_str = result.display(&w);
                        assert_eq!(expected, result_str);
                        assert_eq!(result.passenger, Passenger::new(1, 2));
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
                        assert_eq!(expected0, result0_str);

                        let result1 = result0.apply_action(&w, Actions::West);
                        let result1_str = result1.display(&w);
                        assert_eq!(expected1, result1_str);

                        assert_eq!(result1.passenger, Passenger::new(0, 2));
                    }
                }
            }
        }
    }

}
