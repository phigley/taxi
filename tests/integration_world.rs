extern crate taxi;

use taxi::world::*;
use taxi::position::Position;
use taxi::actions::Actions;

#[test]
fn build_world() {
    let source = "\
    ┌───┬─────┐\n\
    │. .│. . .│\n\
    │   │     │\n\
    │. .│. . .│\n\
    │         │\n\
    │. . . . .│\n\
    │         │\n\
    │.│. .│. .│\n\
    │ │   │   │\n\
    │.│. .│. .│\n\
    └─┴───┴───┘\n\
    ";

    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(world) => {
            assert_eq!(world.width, 5);
            assert_eq!(world.height, 5);
        }
    }
}

#[test]
fn wall_action_validity() {
    let source = "\
    ┌───┬─────┐\n\
    │. .│. . .│\n\
    │   │     │\n\
    │. .│. . .│\n\
    │         │\n\
    │. . . . .│\n\
    │ ┌─      │\n\
    │.│. .│. .│\n\
    │ │   │   │\n\
    │.│. .│. .│\n\
    └─┴───┴───┘\n\
    ";

    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(world) => {

            assert_eq!(
                world.valid_action(&Position::new(2, 2), Actions::North),
                true
            );
            assert_eq!(
                world.valid_action(&Position::new(3, 0), Actions::North),
                false
            );

            assert_eq!(
                world.valid_action(&Position::new(3, 3), Actions::South),
                true
            );
            assert_eq!(
                world.valid_action(&Position::new(1, 2), Actions::South),
                false
            );

            assert_eq!(
                world.valid_action(&Position::new(3, 1), Actions::East),
                true
            );
            assert_eq!(
                world.valid_action(&Position::new(0, 3), Actions::East),
                false
            );

            assert_eq!(
                world.valid_action(&Position::new(1, 1), Actions::West),
                true
            );
            assert_eq!(
                world.valid_action(&Position::new(3, 3), Actions::West),
                false
            );
        }
    }
}

#[test]
fn edge_action_validity() {
    let source = " . . . \n . . . \n . . . \n";

    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(world) => {

            assert_eq!(
                world.valid_action(&Position::new(1, 0), Actions::North),
                false
            );

            assert_eq!(
                world.valid_action(&Position::new(1, 2), Actions::South),
                false
            );

            assert_eq!(
                world.valid_action(&Position::new(2, 1), Actions::East),
                false
            );

            assert_eq!(
                world.valid_action(&Position::new(0, 1), Actions::West),
                false
            );
        }
    }
}

#[test]
fn output_world() {
    let source = "\
    ┌───┬─────┐\n\
    │. .│. . .│\n\
    │   │     │\n\
    │. .│. . .│\n\
    │         │\n\
    │. . . . .│\n\
    │         │\n\
    │.│. .│. .│\n\
    │ │   │   │\n\
    │.│. .│. .│\n\
    └─┴───┴───┘\n\
    ";

    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(world) => {
            let strings = world.display_strings();

            let mut result = String::new();

            for s in strings {
                result += &s;
                result.push('\n');
            }

            println!("\n{}\n{}", result, source);

            assert_eq!(result, source);
        }
    }
}
