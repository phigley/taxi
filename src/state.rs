

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
    fn build_empty() -> State {
        State {
            taxi: Taxi::new(-1, -1),
            passenger: Passenger::new(-1, -1),
            destination: Destination::new(-1, -1),
        }

    }

    pub fn build(
        world: &World,
        taxi_pos: &Position,
        passenger_id: char,
        destination_id: char,
    ) -> Result<State, String> {

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
                            taxi: Taxi::new(taxi_pos.x, taxi_pos.y),
                            passenger: Passenger::new(passenger_pos.x, passenger_pos.y),
                            destination: Destination::new(destination_pos.x, destination_pos.y),
                        })
                    }
                }
            }
        }
    }

    pub fn build_from_str(source: &str, world: &World) -> Result<State, String> {

        let mut lines = source.lines();

        match lines.next() {

            None => Err(String::from("Empty string pased to State::new")),

            Some(_) => {

                let mut result = State::build_empty();

                let mut current_y = 0;

                let mut process_line = true;

                for l in lines {

                    if process_line {
                        parse_line(l, current_y, &world, &mut result)?;
                        current_y += 1;

                    }

                    process_line = !process_line;
                }

                if result.taxi.position.x < 0 || result.taxi.position.y < 0 {
                    return Err(String::from("No taxi found."));
                }

                if result.passenger.position.x < 0 || result.passenger.position.y < 0 {
                    return Err(String::from("No passenger found."));
                }

                if result.destination.position.x < 0 || result.destination.position.y < 0 {
                    return Err(String::from("No destination found."));
                }

                Ok(result)
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

fn parse_line(line: &str, current_y: i32, world: &World, result: &mut State) -> Result<(), String> {

    if current_y >= world.height {
        return Err(format!(
            "Reading line number {} which is greater than world's height of {}.",
            current_y + 1,
            world.height
        ));
    }

    for (i, c) in line.chars().enumerate() {
        if i % 2 == 1 {
            let x = (i / 2) as i32;

            if x >= world.width {
                return Err(format!(
                    "Reading character position {} which is greater than world's width of {}",
                    x,
                    world.width
                ));
            }

            match c {
                '.' => (),

                't' => {

                    if result.taxi.position.x >= 0 || result.taxi.position.y >= 0 {
                        return Err(format!(
                            "Found second taxi at {},{}.  First was at {}, {}",
                            x,
                            current_y,
                            result.taxi.position.x,
                            result.taxi.position.y
                        ));
                    }
                    result.taxi = Taxi::new(x, current_y);
                }
                'T' => {

                    if result.taxi.position.x >= 0 || result.taxi.position.y >= 0 {
                        return Err(format!(
                            "Found second taxi at {},{}.  First was at {}, {}",
                            x,
                            current_y,
                            result.taxi.position.x,
                            result.taxi.position.y
                        ));
                    }
                    if result.passenger.position.x >= 0 || result.passenger.position.y >= 0 {
                        return Err(format!(
                            "Found second passenger at {},{}.  First was at {}, {}",
                            x,
                            current_y,
                            result.passenger.position.x,
                            result.passenger.position.y
                        ));
                    }
                    result.taxi = Taxi::new(x, current_y);
                    result.passenger = Passenger::new(x, current_y);
                }
                'p' => {
                    if result.passenger.position.x >= 0 || result.passenger.position.y >= 0 {
                        return Err(format!(
                            "Found second passenger at {},{}.  First was at {}, {}",
                            x,
                            current_y,
                            result.passenger.position.x,
                            result.passenger.position.y
                        ));
                    }
                    result.passenger = Passenger::new(x, current_y);
                }
                'd' => {
                    if result.destination.position.x >= 0 || result.destination.position.y >= 0 {
                        return Err(format!(
                            "Found second destination at {},{}.  First was at {}, {}",
                            x,
                            current_y,
                            result.destination.position.x,
                            result.destination.position.y
                        ));
                    }
                    result.destination = Destination::new(x, current_y);
                }
                _ => {
                    return Err(format!(
                        "Unknown character {}, at position ({}, {})",
                        char::from(c),
                        x,
                        current_y
                    ))
                }
            }
        }
    }

    Ok(())
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
    fn build_fails_unknown_character() {
        let source = "\
        ┌───────┐\n\
        │. . . .│\n\
        │       │\n\
        │. . < .│\n\
        └───────┘\n\
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
    fn build_correct_empty_walls() {

        let mut source = String::new();
        source += "           \n";
        source += " d . . . . \n";
        source += "           \n";
        source += " . T . . . \n";
        source += "           \n";
        source += " . . . . . \n";
        source += "           \n";
        source += " . . . . . \n";
        source += "           \n";
        source += " . . . . . \n";
        source += "           \n";

        match World::build_from_str(&source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let mut expected_state = State::build_empty();
                expected_state.taxi = Taxi::new(1, 1);
                expected_state.passenger = Passenger::new(1, 1);
                expected_state.destination = Destination::new(0, 0);

                match State::build_from_str(&source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(res_state) => {
                        assert_eq!(res_state, expected_state);
                    }
                }
            }
        }

    }

    #[test]
    fn build_correct_single_walls() {
        let mut source = String::new();
        source += "           \n";
        source += " d . . . . \n";
        source += "           \n";
        source += " .│T . . . \n";
        source += "           \n";
        source += " . . . . . \n";
        source += "           \n";
        source += " . . . . . \n";
        source += "           \n";
        source += " . . . . . \n";
        source += "           \n";

        match World::build_from_str(&source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let mut expected_state = State::build_empty();
                expected_state.taxi = Taxi::new(1, 1);
                expected_state.passenger = Passenger::new(1, 1);
                expected_state.destination = Destination::new(0, 0);

                match State::build_from_str(&source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(res_state) => {
                        assert_eq!(res_state, expected_state);
                    }
                }
            }
        }

    }

    #[test]
    fn build_correct_multi_walls() {
        let source = "\
        ┌───┬─────┐\n\
        │T .│. . .│\n\
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
                let mut expected_state = State::build_empty();

                expected_state.taxi = Taxi::new(0, 0);
                expected_state.passenger = Passenger::new(0, 0);
                expected_state.destination = Destination::new(3, 3);

                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(res_state) => {
                        assert_eq!(res_state, expected_state);
                    }
                }
            }
        }
    }

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
                let mut expected_state = State::build_empty();

                expected_state.taxi = Taxi::new(1, 3);
                expected_state.passenger = Passenger::new(0, 0);
                expected_state.destination = Destination::new(3, 3);

                match State::build(&w, &Position::new(1, 3), 'R', 'B') {
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
                match State::build(&w, &Position::new(1, 3), 'C', 'B') {
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
                match State::build(&w, &Position::new(1, 3), 'Y', 'Q') {
                    Err(_) => (), // panic!(msg),
                    Ok(res_state) => panic!("Found valid destination: {:?}", res_state.destination),
                }
            }
        }
    }
    #[test]
    fn build_correct_outside_taxi() {
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
                let mut expected_state = State::build_empty();

                expected_state.taxi = Taxi::new(1, 3);
                expected_state.passenger = Passenger::new(0, 0);
                expected_state.destination = Destination::new(3, 3);

                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(res_state) => {
                        assert_eq!(res_state, expected_state);
                    }
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
        │.│T .│d .│\n\
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
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let result = state.apply_action(&w, Actions::North);
                        let result_str = result.display(&w);
                        assert_eq!(expected, result_str);
                        assert_eq!(result.passenger, Passenger::new(1,2));
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
        │. p . . .│\n\
        │         │\n\
        │.│t .│d .│\n\
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
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let result0 = state.apply_action(&w, Actions::North);
                        let result0_str = result0.display(&w);
                        assert_eq!(expected0, result0_str);

                        let result1 = result0.apply_action(&w, Actions::West);
                        let result1_str = result1.display(&w);
                        assert_eq!(expected1, result1_str);

                        assert_eq!(result1.passenger, Passenger::new(0,2));
                    }
                }
            }
        }
    }

}
