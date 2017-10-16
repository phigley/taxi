#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate taxi;

use taxi::state::*;
use taxi::world::World;
use taxi::actions::Actions;

#[test]
fn output_matches_str_simple() {
    let mut source = String::new();
    source += "     \n";
    source += " d . \n";
    source += "     \n";
    source += " . T \n";
    source += "     \n";

    match World::build_from_str(&source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(&source, &w) {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let output = state.display(&w);
                    assert_eq!(output, source);
                }
            }
        }
    }

}

#[test]
fn fail_no_taxi() {
    let source = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
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

    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            let res = State::build_from_str(source, &w);
            assert_matches!( res, Err( _ ))
        }
    }
}

#[test]
fn fail_multi_taxi() {
    let source = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │t . . . .│\n\
        │         │\n\
        │.│t .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";
    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            let res = State::build_from_str(source, &w);
            assert_matches!( res, Err( _ ))
        }
    }

}

#[test]
fn fail_multi_taxi_with_passenger() {
    let source = "\
        ┌───┬─────┐\n\
        │T .│. . .│\n\
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
    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            let res = State::build_from_str(source, &w);
            assert_matches!( res, Err( _ ))
        }
    }

}

#[test]
fn fail_no_passenger() {
    let source = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
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
    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            let res = State::build_from_str(source, &w);
            assert_matches!( res, Err( _ ))
        }
    }
}

#[test]
fn fail_multi_passenger() {
    let source = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. p .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│t .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";
    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            let res = State::build_from_str(source, &w);
            assert_matches!( res, Err( _ ))
        }
    }
}

#[test]
fn fail_multi_passenger_in_taxi() {
    let source = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│T .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            let res = State::build_from_str(source, &w);
            assert_matches!( res, Err( _ ))
        }
    }

}

#[test]
fn fail_no_destination() {
    let source = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│t .│. .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";
    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            let res = State::build_from_str(source, &w);
            assert_matches!( res, Err( _ ))
        }
    }
}

#[test]
fn fail_multi_destination() {
    let source = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│t .│d .│\n\
        │ │   │   │\n\
        │d│. .│. .│\n\
        └─┴───┴───┘\n\
        ";
    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            let res = State::build_from_str(source, &w);
            assert_matches!( res, Err( _ ));
        }
    }

}

#[test]
fn output_matches_str_complex() {
    let source = "\
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

    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let state_str = state.display(&w);
                    assert_eq!(source, state_str);
                }
            }
        }
    }
}

#[test]
fn move_allowed_north() {
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
        ┌───┬─────┐\n\
        │p .│. . .│\n\
        │   │     │\n\
        │. .│. t .│\n\
        │         │\n\
        │. . . . .│\n\
        │ ┌─      │\n\
        │.│. .│d .│\n\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
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


    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
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
    let source = "\
        ┌───┬─────┐\n\
        │. .│. . .│\n\
        │   │     │\n\
        │. .│t p .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│d .│\n\
        │ │   │   │\n\
        │.│. .│. .│\n\
        └─┴───┴───┘\n\
        ";

    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(w) => {
            match State::build_from_str(source, &w) {
                Err(msg) => panic!(msg),
                Ok(state) => {
                    let result0 = state.apply_action(&w, Actions::East);
                    assert!( result0.at_destination() == false );
                    let result1 = result0.apply_action(&w, Actions::South);
                    assert!( result1.at_destination() == false );
                    let result2 = result1.apply_action(&w, Actions::South);
                    assert!( result2.at_destination() == true );
                }
            }
        }
    }
}
