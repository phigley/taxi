extern crate taxi;

use taxi::actions::Actions;
use taxi::position::Position;
use taxi::world::*;

#[test]
#[should_panic]
fn build_fails_on_emptystring() {
    World::build_from_str("").unwrap();
}

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
fn wall_move_validity() {
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
                world.determine_affect(&Position::new(2, 2), Actions::North),
                ActionAffect::Move(Position::new(0, -1))
            );
            assert_eq!(
                world.determine_affect(&Position::new(3, 0), Actions::North),
                ActionAffect::Invalid
            );

            assert_eq!(
                world.determine_affect(&Position::new(3, 3), Actions::South),
                ActionAffect::Move(Position::new(0, 1))
            );
            assert_eq!(
                world.determine_affect(&Position::new(1, 2), Actions::South),
                ActionAffect::Invalid
            );

            assert_eq!(
                world.determine_affect(&Position::new(3, 1), Actions::East),
                ActionAffect::Move(Position::new(1, 0))
            );
            assert_eq!(
                world.determine_affect(&Position::new(0, 3), Actions::East),
                ActionAffect::Invalid
            );

            assert_eq!(
                world.determine_affect(&Position::new(1, 1), Actions::West),
                ActionAffect::Move(Position::new(-1, 0))
            );
            assert_eq!(
                world.determine_affect(&Position::new(3, 3), Actions::West),
                ActionAffect::Invalid
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
                world.determine_affect(&Position::new(1, 0), Actions::North),
                ActionAffect::Invalid
            );

            assert_eq!(
                world.determine_affect(&Position::new(1, 2), Actions::South),
                ActionAffect::Invalid
            );

            assert_eq!(
                world.determine_affect(&Position::new(2, 1), Actions::East),
                ActionAffect::Invalid
            );

            assert_eq!(
                world.determine_affect(&Position::new(0, 1), Actions::West),
                ActionAffect::Invalid
            );
        }
    }
}

#[test]
fn read_fixed_position() {
    let source = "\
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

    match World::build_from_str(source) {
        Err(msg) => panic!(msg),
        Ok(world) => {
            assert_eq!(world.num_fixed_positions(), 4);

            assert_eq!(world.get_fixed_position('R'), Some(&Position::new(0, 0)));
            assert_eq!(world.get_fixed_position('G'), Some(&Position::new(4, 0)));
            assert_eq!(world.get_fixed_position('Y'), Some(&Position::new(0, 4)));
            assert_eq!(world.get_fixed_position('B'), Some(&Position::new(3, 4)));

            assert_eq!(world.get_fixed_position('?'), None);
        }
    }
}

#[test]
fn fixed_position_indices() {
    let source = "\
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

    let world = World::build_from_str(source).unwrap();

    let tests = ['R', 'G', 'B', 'Y'];

    for test in &tests {
        println!("Testing '{}'", *test);
        if let Some(index) = world.get_fixed_index(*test) {
            assert_eq!(Some(*test), world.get_fixed_id_from_index(index));
        } else {
            panic!("Index is None");
        }
    }

    println!("Testing '?'");
    assert_eq!(None, world.get_fixed_index('?'));
    assert_eq!(None, world.get_fixed_id_from_index(4));
    assert_eq!(None, world.get_fixed_id_from_index(12));
}

#[test]
#[should_panic(expected = "'R'")]
fn no_duplicate_fixed_position() {
    let source = "\
                  ┌───┬─────┐\n\
                  │R .│. . G│\n\
                  │   │     │\n\
                  │. .│. . .│\n\
                  │         │\n\
                  │. . . . .│\n\
                  │         │\n\
                  │.│. .│. .│\n\
                  │ │   │   │\n\
                  │Y│. .│R .│\n\
                  └─┴───┴───┘\n\
                  ";

    World::build_from_str(source).unwrap();
}

#[test]
fn pickup_dropoff_validity() {
    let source = "\
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

    let world = World::build_from_str(source).unwrap();

    assert_eq!(
        world.determine_affect(&Position::new(0, 0), Actions::PickUp),
        ActionAffect::PickUp('R')
    );

    assert_eq!(
        world.determine_affect(&Position::new(1, 0), Actions::PickUp),
        ActionAffect::Invalid
    );

    assert_eq!(
        world.determine_affect(&Position::new(3, 4), Actions::DropOff),
        ActionAffect::DropOff('B')
    );

    assert_eq!(
        world.determine_affect(&Position::new(2, 4), Actions::DropOff),
        ActionAffect::Invalid
    );
}

#[test]
fn output_world() {
    let source = "\
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

    let world = World::build_from_str(source).unwrap();
    let strings = world.display_strings();

    let mut result = String::new();

    for s in strings {
        result += &s;
        result.push('\n');
    }

    println!("\n{}\n{}", result, source);

    assert_eq!(result, source);
}
