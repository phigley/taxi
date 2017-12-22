extern crate taxi;

use taxi::state_indexer::StateIndexer;
use taxi::world::World;
use taxi::state::State;

#[test]
fn indices_unique() {
    let source_world = "\
                        ┌─────┐\n\
                        │R . G│\n\
                        │     │\n\
                        │. . .│\n\
                        │     │\n\
                        │. Y .│\n\
                        └─────┘\n\
                        ";

    let possible_passengers = [Some('R'), Some('G'), Some('Y'), None];

    let possible_destinations = ['R', 'Y', 'G'];

    let world = World::build_from_str(source_world).unwrap();

    let state_indexer = StateIndexer::new(&world);

    let mut visited_states = vec![false; state_indexer.num_states()];

    for destination in &possible_destinations {
        for passenger in &possible_passengers {
            for y in 0..world.height {
                for x in 0..world.width {
                    let state = State::build(&world, (x, y), *passenger, *destination).unwrap();
                    let state_index = state_indexer.get_index(&world, &state).unwrap();

                    assert!(state_index < visited_states.len());
                    assert!(!visited_states[state_index]);

                    let reconstructed_state = state_indexer.get_state(&world, state_index).unwrap();

                    println!(
                        "---- {} ----\n{}\n{}",
                        state_index,
                        state.display(&world),
                        reconstructed_state.display(&world),
                    );

                    assert_eq!(state.display(&world), reconstructed_state.display(&world));

                    visited_states[state_index] = true;
                }
            }
        }
    }

    for (i, v) in visited_states.iter().enumerate() {
        if !v {
            println!("State index {} not visited.", i);
        }

        assert!(v);
    }
}
