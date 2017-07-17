
use std::string;
use position::Position;
use world::World;

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

#[derive(Debug, PartialEq)]
pub enum Actions {
    North,
    South,
    East,
    West,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct State<'a> {
    world: &'a World,
    taxi: Taxi,
    passenger: Passenger,
    destination: Destination,
}

impl<'a> State<'a> {
    fn build_empty(world: &'a World) -> State<'a> {
        State {
            world: world,
            taxi: Taxi::new(-1, -1),
            passenger: Passenger::new(-1, -1),
            destination: Destination::new(-1, -1),
        }

    }

    pub fn build_from_str(source: &str, world: &'a World) -> Result<State<'a>, String> {

        let mut lines = source.lines();

        match lines.next() {

            None => Err(String::from("Empty string pased to State::new")),

            Some(first_line) => {

                let mut result = State::build_empty(world);

                let mut current_y = 0;
                parse_line(first_line, current_y, &mut result)?;

                for l in lines {

                    current_y += 1;

                    parse_line(l, current_y, &mut result)?;
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



    fn display(&self) -> Result<String, string::FromUtf8Error> {

        let string_width = (self.world.width + 1) as usize;
        let mut bytes = self.world.display_bytes();

        if self.taxi.position.x >= 0 && self.taxi.position.y >= 0 {
            let column_offset = (self.taxi.position.y as usize) * string_width;
            let row_offset = self.taxi.position.x as usize;

            let b: &mut u8 = &mut bytes[column_offset + row_offset];

            *b = if *b == b'.' { b't' } else { b'!' };
        }

        if self.passenger.position.x >= 0 && self.passenger.position.y >= 0 {
            let column_offset = (self.passenger.position.y as usize) * string_width;
            let row_offset = self.passenger.position.x as usize;

            let b: &mut u8 = &mut bytes[column_offset + row_offset];

            *b = if *b == b'.' {
                b'p'
            } else if *b == b't' {
                b'T'
            } else {
                b'!'
            };
        }

        if self.destination.position.x >= 0 && self.destination.position.y >= 0 {
            let column_offset = (self.destination.position.y as usize) * string_width;
            let row_offset = self.destination.position.x as usize;

            let b: &mut u8 = &mut bytes[column_offset + row_offset];

            *b = if *b == b'.' {
                b'd'
            } else if *b == b't' {
                b's'
            } else if *b == b'T' {
                b'D'
            } else {
                b'!'
            };
        }

        String::from_utf8(bytes)
    }

    pub fn apply_action(&self, action: Actions) -> State<'a> {
        let delta = position_delta(action);

        let new_taxi_pos = self.taxi.position + delta;

        if !self.valid_position(new_taxi_pos) {
            *self
        } else {
            let new_taxi = Taxi {
                position: new_taxi_pos,
                ..self.taxi
            };

            State {
                taxi: new_taxi,
                ..*self
            }

        }

    }

    fn valid_position(&self, position: Position) -> bool {
        if position.x < 0 {
            false
        } else if position.x >= self.world.width {
            false
        } else if position.y < 0 {
            false
        } else if position.y >= self.world.height {
            false
        } else {
            for w in self.world.walls.iter() {
                if w.position == position {
                    return false;
                }
            }

            true
        }
    }
}

fn parse_line(line: &str, current_y: i32, result: &mut State) -> Result<(), String> {

    if current_y >= result.world.height {
        return Err(format!(
            "Reading line number {} which is greater than world's height of {}.",
            current_y + 1,
            result.world.height
        ));
    }

    for (i, c) in line.bytes().enumerate() {
        match c {
            b'.' => {
                if i as i32 >= result.world.width {
                    return Err(format!(
                        "Reading character number {} which is greater than world's width of {}",
                        i,
                        result.world.width
                    ));
                }
            }
            b'#' => {
                if i as i32 >= result.world.width {
                    return Err(format!(
                        "Reading character number {} which is greater than world's width of {}",
                        i,
                        result.world.width
                    ));
                }
            }
            b't' => {
                if i as i32 >= result.world.width {
                    return Err(format!(
                        "Reading character number {} which is greater than world's width of {}",
                        i,
                        result.world.width
                    ));
                } else if result.taxi.position.x >= 0 || result.taxi.position.y >= 0 {
                    return Err(format!(
                        "Found second taxi at {},{}.  First was at {}, {}",
                        i as i32,
                        current_y,
                        result.taxi.position.x,
                        result.taxi.position.y
                    ));
                }
                result.taxi = Taxi::new(i as i32, current_y);
            }
            b'T' => {
                if i as i32 >= result.world.width {
                    return Err(format!(
                        "Reading character number {} which is greater than world's width of {}",
                        i,
                        result.world.width
                    ));
                } else if result.taxi.position.x >= 0 || result.taxi.position.y >= 0 {
                    return Err(format!(
                        "Found second taxi at {},{}.  First was at {}, {}",
                        i as i32,
                        current_y,
                        result.taxi.position.x,
                        result.taxi.position.y
                    ));
                }
                if result.passenger.position.x >= 0 || result.passenger.position.y >= 0 {
                    return Err(format!(
                        "Found second passenger at {},{}.  First was at {}, {}",
                        i as i32,
                        current_y,
                        result.passenger.position.x,
                        result.passenger.position.y
                    ));
                }
                result.taxi = Taxi::new(i as i32, current_y);
                result.passenger = Passenger::new(i as i32, current_y);
            }
            b'p' => {
                if i as i32 >= result.world.width {
                    return Err(format!(
                        "Reading character number {} which is greater than world's width of {}",
                        i,
                        result.world.width
                    ));
                } else if result.passenger.position.x >= 0 || result.passenger.position.y >= 0 {
                    return Err(format!(
                        "Found second passenger at {},{}.  First was at {}, {}",
                        i as i32,
                        current_y,
                        result.passenger.position.x,
                        result.passenger.position.y
                    ));
                }
                result.passenger = Passenger::new(i as i32, current_y);
            }
            b'd' => {
                if i as i32 >= result.world.width {
                    return Err(format!(
                        "Reading character number {} which is greater than world's width of {}",
                        i,
                        result.world.width
                    ));
                } else if result.destination.position.x >= 0 || result.destination.position.y >= 0 {
                    return Err(format!(
                        "Found second destination at {},{}.  First was at {}, {}",
                        i as i32,
                        current_y,
                        result.destination.position.x,
                        result.destination.position.y
                    ));
                }
                result.destination = Destination::new(i as i32, current_y);
            }
            _ => {
                return Err(format!(
                    "Unknown character {}, line {}, col {}",
                    char::from(c),
                    current_y,
                    i
                ))
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
        ....\n\
        ..<.\n\
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
        let source = "\
        d....\n\
        .T...\n\
        .....\n\
        .....\n\
        .....\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let mut expected_state = State::build_empty(&w);
                expected_state.taxi = Taxi::new(1, 1);
                expected_state.passenger = Passenger::new(1, 1);
                expected_state.destination = Destination::new(0, 0);

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
    fn build_correct_single_walls() {
        let source = "\
        d....\n\
        .T...\n\
        ...#.\n\
        .....\n\
        .....\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let mut expected_state = State::build_empty(&w);
                expected_state.taxi = Taxi::new(1, 1);
                expected_state.passenger = Passenger::new(1, 1);
                expected_state.destination = Destination::new(0, 0);

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
    fn build_correct_multi_walls() {
        let source = "\
        ##########\n\
        #T..#....#\n\
        #...#....#\n\
        #.....#..#\n\
        #.#...#d.#\n\
        #.#...#..#\n\
        ##########\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let mut expected_state = State::build_empty(&w);

                expected_state.taxi = Taxi::new(1, 1);
                expected_state.passenger = Passenger::new(1, 1);
                expected_state.destination = Destination::new(7, 4);

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
    fn build_correct_taxi() {
        let source = "\
        d....\n\
        .....\n\
        .....\n\
        .T...\n\
        .....\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let mut expected_state = State::build_empty(&w);
                expected_state.taxi = Taxi::new(1, 3);
                expected_state.passenger = Passenger::new(1, 3);
                expected_state.destination = Destination::new(0, 0);

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
    fn build_correct_passenger() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        .t...\n\
        .....\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let mut expected_state = State::build_empty(&w);
                expected_state.taxi = Taxi::new(1, 3);
                expected_state.passenger = Passenger::new(3, 1);
                expected_state.destination = Destination::new(0, 0);

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
    fn build_correct_destination() {
        let source = "\
        .....\n\
        ...p.\n\
        .....\n\
        .t...\n\
        ...d.\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let mut expected_state = State::build_empty(&w);
                expected_state.taxi = Taxi::new(1, 3);
                expected_state.passenger = Passenger::new(3, 1);
                expected_state.destination = Destination::new(3, 4);

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
    fn build_passenger_in_taxi() {
        let source = "\
        .....\n\
        .....\n\
        .....\n\
        .T...\n\
        ...d.\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                let mut expected_state = State::build_empty(&w);
                expected_state.taxi = Taxi::new(1, 3);
                expected_state.passenger = Passenger::new(1, 3);
                expected_state.destination = Destination::new(3, 4);

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
    fn fail_no_taxi() {
        let source = "\
        .....\n\
        ...p.\n\
        .....\n\
        .....\n\
        ...d.\n\
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
        .....\n\
        ...t.\n\
        .....\n\
        .....\n\
        ...t.\n\
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
        .....\n\
        ...t.\n\
        .....\n\
        .....\n\
        ...T.\n\
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
        .....\n\
        ...t.\n\
        .....\n\
        .....\n\
        ...d.\n\
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
        .....\n\
        ...p.\n\
        .....\n\
        .....\n\
        ..pt.\n\
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
        .....\n\
        ...p.\n\
        .....\n\
        .....\n\
        ...T.\n\
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
        .....\n\
        ...t.\n\
        .....\n\
        .....\n\
        ...p.\n\
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
        .....\n\
        ...d.\n\
        .....\n\
        .....\n\
        ..dT.\n\
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
    fn output_matches_str_simple() {
        let source = "\
        d.\n\
        .T\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let output = state.display();
                        match output {
                            Err(format_msg) => panic!("{:?}", format_msg),
                            Ok(output_str) => assert_eq!(output_str, source),
                        }

                    }
                }
            }
        }

    }

    fn test_expected(expected_str: &str, result: &State) {
        match result.display() {
            Err(format_msg) => panic!(format_msg),
            Ok(result_str) => assert_eq!(expected_str, result_str),
        }
    }

    #[test]
    fn output_matches_str_complex() {
        let source = "\
        ##########\n\
        #T..#....#\n\
        #...#....#\n\
        #.....#..#\n\
        #.#...#d.#\n\
        #.#...#..#\n\
        ##########\n\
        ";

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => test_expected(source, &state),
                }
            }
        }
    }

    #[test]
    fn move_allowed_north() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        .t...\n\
        .....\n\
        ";

        let expected_north = "\
        d....\n\
        ...p.\n\
        .t...\n\
        .....\n\
        .....\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_north = state.apply_action(Actions::North);
                        test_expected(expected_north, &state_north);
                    }
                }
            }
        }
    }

    #[test]
    fn move_top_north() {
        let source = "\
        dt...\n\
        ...p.\n\
        .....\n\
        .....\n\
        .....\n\
        ";

        let expected_north = "\
        dt...\n\
        ...p.\n\
        .....\n\
        .....\n\
        .....\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_north = state.apply_action(Actions::North);
                        test_expected(expected_north, &state_north);
                    }
                }
            }
        }
    }

    #[test]
    fn move_wall_north() {
        let source = "\
        d....\n\
        ...p.\n\
        .#...\n\
        .t...\n\
        .....\n\
        ";

        let expected_north = "\
        d....\n\
        ...p.\n\
        .#...\n\
        .t...\n\
        .....\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_north = state.apply_action(Actions::North);
                        test_expected(expected_north, &state_north);
                    }
                }
            }
        }
    }

    #[test]
    fn move_allowed_south() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        .t...\n\
        .....\n\
        ";

        let expected_south = "\
        d....\n\
        ...p.\n\
        .....\n\
        .....\n\
        .t...\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_south = state.apply_action(Actions::South);
                        test_expected(expected_south, &state_south);
                    }
                }
            }
        }
    }

    #[test]
    fn move_bottom_south() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        .....\n\
        .t...\n\
        ";

        let expected_south = "\
        d....\n\
        ...p.\n\
        .....\n\
        .....\n\
        .t...\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_south = state.apply_action(Actions::South);
                        test_expected(expected_south, &state_south);
                    }
                }
            }
        }
    }

    #[test]
    fn move_wall_south() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        .t...\n\
        .#...\n\
        ";

        let expected_south = "\
        d....\n\
        ...p.\n\
        .....\n\
        .t...\n\
        .#...\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_south = state.apply_action(Actions::South);
                        test_expected(expected_south, &state_south);
                    }
                }
            }
        }
    }

    #[test]
    fn move_allowed_east() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        .t...\n\
        .....\n\
        ";

        let expected_east = "\
        d....\n\
        ...p.\n\
        .....\n\
        ..t..\n\
        .....\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_east = state.apply_action(Actions::East);
                        test_expected(expected_east, &state_east);
                    }
                }
            }
        }
    }

    #[test]
    fn move_right_east() {
        let source = "\
        d....\n\
        ...p.\n\
        ....t\n\
        .....\n\
        .....\n\
        ";

        let expected_east = "\
        d....\n\
        ...p.\n\
        ....t\n\
        .....\n\
        .....\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_east = state.apply_action(Actions::East);
                        test_expected(expected_east, &state_east);
                    }
                }
            }
        }
    }

    #[test]
    fn move_wall_east() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        .t#..\n\
        .....\n\
        ";

        let expected_east = "\
        d....\n\
        ...p.\n\
        .....\n\
        .t#..\n\
        .....\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_east = state.apply_action(Actions::East);
                        test_expected(expected_east, &state_east);
                    }
                }
            }
        }
    }

    #[test]
    fn move_allowed_west() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        .t...\n\
        .....\n\
        ";

        let expected_west = "\
        d....\n\
        ...p.\n\
        .....\n\
        t....\n\
        .....\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_west = state.apply_action(Actions::West);
                        test_expected(expected_west, &state_west);
                    }
                }
            }
        }
    }

    #[test]
    fn move_left_west() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        t....\n\
        .....\n\
        ";

        let expected_west = "\
        d....\n\
        ...p.\n\
        .....\n\
        t....\n\
        .....\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_west = state.apply_action(Actions::West);
                        test_expected(expected_west, &state_west);
                    }
                }
            }
        }
    }

    #[test]
    fn move_wall_west() {
        let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        #t...\n\
        .....\n\
        ";

        let expected_west = "\
        d....\n\
        ...p.\n\
        .....\n\
        #t...\n\
        .....\n\
        ";


        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                match State::build_from_str(source, &w) {
                    Err(msg) => panic!(msg),
                    Ok(state) => {
                        let state_west = state.apply_action(Actions::West);
                        test_expected(expected_west, &state_west);
                    }
                }
            }
        }
    }
}
