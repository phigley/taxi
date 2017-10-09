extern crate taxi;

use taxi::world::*;
use taxi::position::Position;

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
fn get_wall() {
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

            let w = world.get_wall(&Position::new(1, 3));

            assert!(w.north);
            assert!(!w.south);
            assert!(!w.east);
            assert!(w.west);
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
