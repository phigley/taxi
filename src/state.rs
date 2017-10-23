

use std::fmt;

use rand::Rng;

use position::Position;
use world::{World, ActionAffect};
use actions::Actions;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct State {
    taxi: Position,
    passenger: Option<char>,
    destination: char,
}

pub enum Error {
    InvalidTaxi {
        taxi_pos: (i32, i32),
        world_dims: (i32, i32),
    },

    InvalidDestination { id: char, world: String },

    InvalidPassenger { id: char, world: String },

    TooFewFixedPositions {
        num_fixed_positions: usize,
        world: String,
    },
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidTaxi {
                taxi_pos,
                world_dims,
            } => {
                write!(
                    f,
                    "Taxi position ({},{}) is invalid, world (width, height) is ({},{}).",
                    taxi_pos.0,
                    taxi_pos.1,
                    world_dims.0,
                    world_dims.1
                )
            }

            Error::InvalidDestination { id, ref world } => {
                write!(
                    f,
                    "Failed to find destination location '{}' in world:\n{}",
                    id,
                    world
                )
            }

            Error::InvalidPassenger { id, ref world } => {
                write!(
                    f,
                    "Failed to find passenger location '{}' in world:\n{}",
                    id,
                    world
                )
            }

            Error::TooFewFixedPositions {
                num_fixed_positions,
                ref world,
            } => {
                write!(
                    f,
                    "World does not have enough fixed positions. \
                     Need at least 2, but only have {} in world:\n{}",
                    num_fixed_positions,
                    world,
                )
            }
        }
    }
}

impl State {
    pub fn build(
        world: &World,
        taxi_pos: (i32, i32),
        passenger_id: char,
        destination_id: char,
    ) -> Result<State, Error> {

        if taxi_pos.0 < 0 || taxi_pos.0 >= world.width || taxi_pos.1 < 0 ||
            taxi_pos.1 >= world.height
        {
            Err(Error::InvalidTaxi {
                taxi_pos,
                world_dims: (world.width, world.height),
            })
        } else {

            if world.get_fixed_position(destination_id) == None {
                Err(Error::InvalidDestination {
                    id: destination_id,
                    world: world.display(),
                })
            } else if world.get_fixed_position(passenger_id) == None {
                Err(Error::InvalidPassenger {
                    id: passenger_id,
                    world: world.display(),
                })
            } else {
                Ok(State {
                    taxi: Position::new(taxi_pos.0, taxi_pos.1),
                    passenger: Some(passenger_id),
                    destination: destination_id,
                })
            }
        }
    }

    pub fn build_random<R: Rng>(world: &World, rng: &mut R) -> Result<State, Error> {

        let taxi_x = rng.gen_range(0, world.width);
        let taxi_y = rng.gen_range(0, world.height);

        let num_fixed_positions = world.fixed_positions.len();

        if num_fixed_positions < 2 {
            return Err(Error::TooFewFixedPositions {
                num_fixed_positions,
                world: world.display(),
            });
        }

        let passenger_fp_index = rng.gen_range(0, num_fixed_positions);
        let destination_fp_index = (passenger_fp_index + rng.gen_range(1, num_fixed_positions)) %
            num_fixed_positions;

        Ok(State {
            taxi: Position::new(taxi_x, taxi_y),
            passenger: Some(world.fixed_positions[passenger_fp_index].id),
            destination: world.fixed_positions[destination_fp_index].id,
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

        match world.determine_affect(&self.taxi, action) {
            ActionAffect::Invalid => *self,
            ActionAffect::Move(delta) => State {
                taxi: self.taxi + delta,
                ..*self
            },

            ActionAffect::PickUp(id) => {
                if self.passenger == Some(id) {
                    State {
                        passenger: None,
                        ..*self
                    }
                } else {
                    *self
                }
            }

            ActionAffect::DropOff(id) => {
                if self.passenger == None {
                    State {
                        passenger: Some(id),
                        ..*self
                    }
                } else {
                    *self
                }
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

        let w = World::build_from_str(source_world).unwrap();
        let expected_state = State {
            taxi: Position::new(1, 3),
            passenger: Some('R'),
            destination: 'B',
        };

        let res_state = State::build(&w, (1, 3), 'R', 'B').unwrap();
        assert_eq!(res_state, expected_state);
    }

    #[test]
    fn pickup_dropoff_does_nothing_off_fixedpoint() {

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

        let w = World::build_from_str(source_world).unwrap();

        let initial_state = State::build(&w, (2, 2), 'R', 'G').unwrap();

        let expected_initial = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│d . .│\n\
        │         │\n\
        │. . t . .│\n\
        │         │\n\
        │.│. .│. .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        assert_eq!(expected_initial, initial_state.display(&w));

        let state0 = initial_state.apply_action(&w, Actions::PickUp);
        assert_eq!(expected_initial, state0.display(&w));

        let state1 = state0.apply_action(&w, Actions::DropOff);
        assert_eq!(expected_initial, state1.display(&w));
    }

    #[test]
    fn pickup_dropoff_does_nothing_on_empty_fixedpoint() {

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

        let w = World::build_from_str(source_world).unwrap();

        let initial_state = State::build(&w, (1, 3), 'R', 'G').unwrap();

        let expected_initial = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│d . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│t .│. .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

        assert_eq!(expected_initial, initial_state.display(&w));

        let state0 = initial_state.apply_action(&w, Actions::PickUp);
        assert_eq!(expected_initial, state0.display(&w));

        let state1 = state0.apply_action(&w, Actions::DropOff);
        assert_eq!(expected_initial, state1.display(&w));
    }

    #[test]
    fn passenger_dropoff_at_other_fixed_position() {
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

        let script = [
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. t . . .│\n\
        │         │\n\
        │.│p .│. .│\n\
        │ │   │   │\n\
        │.│d .│. .│\n\
        └─┴───┴───┘\n\
        ",
                Some('R'),
                false,
                Actions::South,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│p .│. .│\n\
        │ │   │   │\n\
        │.│d .│. .│\n\
        └─┴───┴───┘\n\
        ",
                Some('R'),
                false,
                Actions::PickUp,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│T .│. .│\n\
        │ │   │   │\n\
        │.│d .│. .│\n\
        └─┴───┴───┘\n\
        ",
                None,
                false,
                Actions::East,
            ),
            (
                "\
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
        ",
                None,
                false,
                Actions::DropOff,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. p│. .│\n\
        │ │   │   │\n\
        │.│d .│. .│\n\
        └─┴───┴───┘\n\
        ",
                Some('Y'),
                false,
                Actions::North,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . t . .│\n\
        │         │\n\
        │.│. p│. .│\n\
        │ │   │   │\n\
        │.│d .│. .│\n\
        └─┴───┴───┘\n\
        ",
                Some('Y'),
                false,
                Actions::North,
            ),
        ];

        let w = World::build_from_str(source).unwrap();

        let mut state = State::build(&w, (1, 2), 'R', 'G').unwrap();
        println!("");

        for &(expected_str, expected_passenger, expected_at_destination, next_action) in
            script.iter()
        {
            println!(
                "{} passenger = {:?} at_destination = {:?} next_action = {}",
                state.display(&w),
                state.passenger,
                state.at_destination(),
                next_action
            );

            assert_eq!(expected_passenger, state.passenger);
            assert_eq!(expected_at_destination, state.at_destination());
            assert_eq!(expected_str, state.display(&w));

            state = state.apply_action(&w, next_action);
        }
    }

    #[test]
    fn taxi_full_cycle() {
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

        let script = [
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. p . . .│\n\
        │         │\n\
        │.│t .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ",
                Some('R'),
                false,
                Actions::North,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. p . . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ",
                Some('R'),
                false,
                Actions::PickUp,
            ),
            (
                "\
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
        ",
                None,
                false,
                Actions::East,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . T . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ",
                None,
                false,
                Actions::DropOff,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . T . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ",
                None,
                false,
                Actions::East,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . T .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ",
                None,
                false,
                Actions::PickUp,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . T .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ",
                None,
                false,
                Actions::South,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ",
                None,
                false,
                Actions::DropOff,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│D .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ",
                Some('G'),
                true,
                Actions::East,
            ),
            (
                "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│D t│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ",
                Some('G'),
                true,
                Actions::South,
            ),
        ];

        let w = World::build_from_str(source).unwrap();

        let mut state = State::build(&w, (1, 3), 'R', 'G').unwrap();
        println!("");

        for &(expected_str, expected_passenger, expected_at_destination, next_action) in
            script.iter()
        {
            println!(
                "{} passenger = {:?} at_destination = {:?} next_action = {}",
                state.display(&w),
                state.passenger,
                state.at_destination(),
                next_action
            );

            assert_eq!(expected_passenger, state.passenger);
            assert_eq!(expected_at_destination, state.at_destination());
            assert_eq!(expected_str, state.display(&w));

            state = state.apply_action(&w, next_action);
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
