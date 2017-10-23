#[cfg(test)]

extern crate taxi;

use taxi::state::*;
use taxi::world::World;
use taxi::actions::Actions;

#[test]
fn output_matches_str_simple() {
    let mut source_world = String::new();
    source_world += "     \n";
    source_world += " R . \n";
    source_world += "     \n";
    source_world += " . G \n";
    source_world += "     \n";

    let mut expected_str = String::new();
    expected_str += "     \n";
    expected_str += " d . \n";
    expected_str += "     \n";
    expected_str += " t p \n";
    expected_str += "     \n";

    match World::build_from_str(&source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (0, 1), 'G', 'R') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let output = state.display(&w);
                    assert_eq!(output, expected_str);
                }
            }
        }
    }

}


#[test]
fn output_matches_str_complex() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│B .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_str = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│t .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (1, 3), 'R', 'B') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_str = state.display(&w);
                    assert_eq!(expected_str, state_str);
                }
            }
        }
    }
}

#[test]
fn move_allowed_north() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_north = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. t . . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (1, 3), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_north = state.apply_action(&w, Actions::North);
                    assert_eq!(expected_north, state_north.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_top_north() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_north = "\
        ┌───┬─────┐\n\
        │p t│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (1, 0), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_north = state.apply_action(&w, Actions::North);
                    assert_eq!(expected_north, state_north.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_wall_north() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_north = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│t .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (1, 3), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_north = state.apply_action(&w, Actions::North);
                    assert_eq!(expected_north, state_north.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_allowed_south() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_south = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . t .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (3, 1), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_south = state.apply_action(&w, Actions::South);
                    assert_eq!(expected_south, state_south.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_bottom_south() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_south = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │t│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (0, 4), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_south = state.apply_action(&w, Actions::South);
                    assert_eq!(expected_south, state_south.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_wall_south() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_south = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. t . . .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (1, 2), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_south = state.apply_action(&w, Actions::South);
                    assert_eq!(expected_south, state_south.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_allowed_east() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_east = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . t . .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (1, 2), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_east = state.apply_action(&w, Actions::East);
                    assert_eq!(expected_east, state_east.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_right_east() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_east = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . t│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (3, 1), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_east = state.apply_action(&w, Actions::East);
                    assert_eq!(expected_east, state_east.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_wall_east() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_east = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. t│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (1, 1), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_east = state.apply_action(&w, Actions::East);
                    assert_eq!(expected_east, state_east.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_allowed_west() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_west = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │t .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (1, 1), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_west = state.apply_action(&w, Actions::West);
                    assert_eq!(expected_west, state_west.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_left_west() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_west = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │t . . . .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (1, 2), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_west = state.apply_action(&w, Actions::West);
                    assert_eq!(expected_west, state_west.display(&w));
                }
            }
        }
    }
}

#[test]
fn move_wall_west() {
    let source_world = "\
        ┌───┬─────┐\n\
        │R .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    let expected_west = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│t .│\n\
        └─┴───┴───┘\n\
        ";


    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (3, 4), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_west = state.apply_action(&w, Actions::West);
                    assert_eq!(expected_west, state_west.display(&w));
                }
            }
        }
    }
}

#[test]
fn reaches_destination() {
    let source_world = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│. R .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│G .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    // ┌───┬─────┐
    // │. .│. . .│
    // │   │     │
    // │. .│t p .│
    // │         │
    // │. . . . .│
    // │         │
    // │.│. .│d .│
    // │ │   │   │
    // │.│. .│. .│
    // └─┴───┴───┘

    match World::build_from_str(source_world) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build(&w, (2, 1), 'R', 'G') {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    println!("");
                    println!("{}", state.display(&w));

                    let result0 = state.apply_action(&w, Actions::East);
                    println!("0:\n{}", result0.display(&w));
                    assert!(result0.at_destination() == false);

                    let result1 = result0.apply_action(&w, Actions::PickUp);
                    println!("1:\n{}", result1.display(&w));
                    assert!(result1.at_destination() == false);

                    let result2 = result1.apply_action(&w, Actions::South);
                    println!("2:\n{}", result2.display(&w));
                    assert!(result2.at_destination() == false);

                    let result3 = result2.apply_action(&w, Actions::South);
                    println!("3:\n{}", result3.display(&w));
                    assert!(result3.at_destination() == false);

                    let result4 = result3.apply_action(&w, Actions::DropOff);
                    println!("4:\n{}", result4.display(&w));
                    assert!(result4.at_destination() == true);
                }
            }
        }
    }
}
