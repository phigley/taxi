use std::fmt;

use enum_map::EnumMap;

use crate::state::State;
use crate::world::World;

use crate::doormax::term::Term;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Condition(pub EnumMap<Term, bool>);

impl Condition {
    pub fn new(world: &World, state: &State) -> Self {
        let taxi_pos = state.get_taxi();
        let walls = world.get_wall(taxi_pos);

        let on_passenger = if let Some(passenger_id) = state.get_passenger() {
            if let Some(passenger_pos) = world.get_fixed_position(passenger_id) {
                passenger_pos == taxi_pos
            } else {
                false
            }
        } else {
            false
        };

        let on_destination =
            if let Some(destination_pos) = world.get_fixed_position(state.get_destination()) {
                destination_pos == taxi_pos
            } else {
                false
            };

        let enum_map = enum_map! {
            Term::TouchWallN => walls.north,
            Term::TouchWallS => walls.south,
            Term::TouchWallE => walls.east,
            Term::TouchWallW => walls.west,

            Term::OnPassenger => on_passenger,
            Term::OnDestination => on_destination,
            Term::HasPassenger => state.get_passenger().is_none(),
        };

        Condition(enum_map)
    }

    pub fn enumerate_all() -> Vec<Condition> {
        let num_conditions = 2_usize.pow(7);
        let mut result = Vec::with_capacity(num_conditions);

        let mut accumulator = enum_map! {
            Term::TouchWallN => false,
            Term::TouchWallS => false,
            Term::TouchWallE => false,
            Term::TouchWallW => false,
            Term::OnPassenger => false,
            Term::OnDestination => false,
            Term::HasPassenger => false,
        };

        loop {
            result.push(Condition(accumulator));

            if !accumulator[Term::TouchWallN] {
                accumulator[Term::TouchWallN] = true;
            } else if !accumulator[Term::TouchWallS] {
                accumulator[Term::TouchWallN] = false;
                accumulator[Term::TouchWallS] = true;
            } else if !accumulator[Term::TouchWallE] {
                accumulator[Term::TouchWallN] = false;
                accumulator[Term::TouchWallS] = false;
                accumulator[Term::TouchWallE] = true;
            } else if !accumulator[Term::TouchWallW] {
                accumulator[Term::TouchWallN] = false;
                accumulator[Term::TouchWallS] = false;
                accumulator[Term::TouchWallE] = false;
                accumulator[Term::TouchWallW] = true;
            } else if !accumulator[Term::OnPassenger] {
                accumulator[Term::TouchWallN] = false;
                accumulator[Term::TouchWallS] = false;
                accumulator[Term::TouchWallE] = false;
                accumulator[Term::TouchWallW] = false;
                accumulator[Term::OnPassenger] = true;
            } else if !accumulator[Term::OnDestination] {
                accumulator[Term::TouchWallN] = false;
                accumulator[Term::TouchWallS] = false;
                accumulator[Term::TouchWallE] = false;
                accumulator[Term::TouchWallW] = false;
                accumulator[Term::OnPassenger] = false;
                accumulator[Term::OnDestination] = true;
            } else if !accumulator[Term::HasPassenger] {
                accumulator[Term::TouchWallN] = false;
                accumulator[Term::TouchWallS] = false;
                accumulator[Term::TouchWallE] = false;
                accumulator[Term::TouchWallW] = false;
                accumulator[Term::OnPassenger] = false;
                accumulator[Term::OnDestination] = false;
                accumulator[Term::HasPassenger] = true;
            } else {
                break;
            }
        }

        result
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Condition(ref cond_map) = self;

        fn show_bool(b: bool) -> &'static str {
            if b {
                "1"
            } else {
                "0"
            }
        }

        write!(
            f,
            "Condition({}{}{}{} {}{}{})",
            show_bool(cond_map[Term::TouchWallN]),
            show_bool(cond_map[Term::TouchWallS]),
            show_bool(cond_map[Term::TouchWallE]),
            show_bool(cond_map[Term::TouchWallW]),
            show_bool(cond_map[Term::OnPassenger]),
            show_bool(cond_map[Term::OnDestination]),
            show_bool(cond_map[Term::HasPassenger]),
        )
    }
}

#[cfg(test)]
mod condition_test {

    use super::*;
    use crate::world::Costs;

    #[test]
    fn enumerates_all() {
        let all = Condition::enumerate_all();

        assert_eq!(all.len(), 2_usize.pow(7));

        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert!(all[i] != all[j]);
            }
        }
    }

    #[test]
    fn can_build() {
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
        let costs = Costs::default();
        let w = World::build_from_str(source, costs).unwrap();

        let state0 = State::build(&w, (1, 2), Some('R'), 'G').unwrap();
        let cond0 = Condition::new(&w, &state0);
        let expected_cond0 = Condition(enum_map! {
            Term::TouchWallN => false,
            Term::TouchWallS => false,
            Term::TouchWallE => false,
            Term::TouchWallW => false,
            Term::OnPassenger => false,
            Term::OnDestination => false,
            Term::HasPassenger => false,
        });
        assert_eq!(cond0, expected_cond0);

        let state1 = State::build(&w, (1, 3), Some('R'), 'G').unwrap();
        let cond1 = Condition::new(&w, &state1);
        let expected_cond1 = Condition(enum_map! {
            Term::TouchWallN => false,
            Term::TouchWallS => false,
            Term::TouchWallE => false,
            Term::TouchWallW => true,
            Term::OnPassenger => true,
            Term::OnDestination => false,
            Term::HasPassenger => false,
        });
        assert_eq!(cond1, expected_cond1);

        let state2 = State::build(&w, (1, 4), None, 'G').unwrap();
        let cond2 = Condition::new(&w, &state2);
        let expected_cond2 = Condition(enum_map! {
            Term::TouchWallN => false,
            Term::TouchWallS => true,
            Term::TouchWallE => false,
            Term::TouchWallW => true,
            Term::OnPassenger => false,
            Term::OnDestination => true,
            Term::HasPassenger => true,
        });
        assert_eq!(cond2, expected_cond2);

        let state3 = State::build(&w, (4, 0), None, 'G').unwrap();
        let cond3 = Condition::new(&w, &state3);
        let expected_cond3 = Condition(enum_map! {
            Term::TouchWallN => true,
            Term::TouchWallS => false,
            Term::TouchWallE => true,
            Term::TouchWallW => false,
            Term::OnPassenger => false,
            Term::OnDestination => false,
            Term::HasPassenger => true,
        });
        assert_eq!(cond3, expected_cond3);
    }
}
