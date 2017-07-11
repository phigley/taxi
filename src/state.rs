
use std::string;

#[derive(Debug, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Position {
        Position { x: x, y: y }
    }
}

#[derive(Debug, PartialEq)]
struct Taxi {
    position: Position,
}

impl Taxi {
    fn new(x: i32, y: i32) -> Taxi {
        Taxi { position: Position::new(x, y) }
    }
}

#[derive(Debug, PartialEq)]
struct Passenger {
    position: Position,
}

impl Passenger {
    fn new(x: i32, y: i32) -> Passenger {
        Passenger { position: Position::new(x, y) }
    }
}

#[derive(Debug, PartialEq)]
struct Destination {
    position: Position,
}

impl Destination {
    fn new(x: i32, y: i32) -> Destination {
        Destination { position: Position::new(x, y) }
    }
}

#[derive(Debug, PartialEq)]
struct Wall {
    position: Position,
}


impl Wall {
    fn new(x: i32, y: i32) -> Wall {
        Wall { position: Position::new(x, y) }
    }
}

#[derive(Debug, PartialEq)]
pub struct State {
    width: i32,
    height: i32,
    walls: Vec<Wall>,
    taxi: Taxi,
    passenger: Passenger,
    destination: Destination,
}

impl State {
    fn build_empty() -> State {
        State {
            width: 0,
            height: 0,
            walls: vec![],
            taxi: Taxi::new(-1, -1),
            passenger: Passenger::new(-1, -1),
            destination: Destination::new(-1, -1),
        }

    }

    pub fn build_from_str(source: &str) -> Result<State, String> {

        let mut lines = source.lines();

        match lines.next() {

            None => Err(String::from("Empty string pased to State::new")),

            Some(first_line) => {

                let mut result = State::build_empty();
                result.width = first_line.len() as i32;
                result.height = 1;

                parse_line(first_line, &mut result)?;

                for l in lines {

                    result.height += 1;

                    parse_line(l, &mut result)?;

                    let current_width = l.len();
                    if result.width != current_width as i32 {
                        return Err(format!(
                            "Failed to create state, line {} has width {} expected {}",
                            result.height - 1,
                            current_width,
                            result.width,
                        ));
                    }
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

        let string_width = (self.width + 1) as usize;
        let line_count = self.height as usize;
        let mut bytes = Vec::with_capacity(string_width * line_count);

        for _ in 0..line_count {
            for _ in 0..(string_width - 1) {
                bytes.push(b'.');
            }

            bytes.push(b'\n');
        }

        for w in self.walls.iter() {
            let column_offset = (w.position.y as usize) * string_width;
            let row_offset = w.position.x as usize;

            let b: &mut u8 = &mut bytes[column_offset + row_offset];


            *b = if *b == b'.' { b'#' } else { b'!' };
        }

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
}

fn parse_line(line: &str, result: &mut State) -> Result<(), String> {

    for (i, c) in line.bytes().enumerate() {
        match c {
            b'.' => (),
            b'#' => {
                let w = Wall::new(i as i32, result.height - 1);
                result.walls.push(w);
            }
            b't' => {
                if result.taxi.position.x >= 0 || result.taxi.position.y >= 0 {
                    return Err(format!(
                        "Found second taxi at {},{}.  First was at {}, {}",
                        i as i32,
                        result.height - 1,
                        result.taxi.position.x,
                        result.taxi.position.y
                    ));
                }
                result.taxi = Taxi::new(i as i32, result.height - 1);
            }
            b'T' => {
                if result.taxi.position.x >= 0 || result.taxi.position.y >= 0 {
                    return Err(format!(
                        "Found second taxi at {},{}.  First was at {}, {}",
                        i as i32,
                        result.height - 1,
                        result.taxi.position.x,
                        result.taxi.position.y
                    ));
                }
                if result.passenger.position.x >= 0 || result.passenger.position.y >= 0 {
                    return Err(format!(
                        "Found second passenger at {},{}.  First was at {}, {}",
                        i as i32,
                        result.height - 1,
                        result.passenger.position.x,
                        result.passenger.position.y
                    ));
                }
                result.taxi = Taxi::new(i as i32, result.height - 1);
                result.passenger = Passenger::new(i as i32, result.height - 1);
            }
            b'p' => {
                if result.passenger.position.x >= 0 || result.passenger.position.y >= 0 {
                    return Err(format!(
                        "Found second passenger at {},{}.  First was at {}, {}",
                        i as i32,
                        result.height - 1,
                        result.passenger.position.x,
                        result.passenger.position.y
                    ));
                }
                result.passenger = Passenger::new(i as i32, result.height - 1);
            }
            b'd' => {
                if result.destination.position.x >= 0 || result.destination.position.y >= 0 {
                    return Err(format!(
                        "Found second destination at {},{}.  First was at {}, {}",
                        i as i32,
                        result.height - 1,
                        result.destination.position.x,
                        result.destination.position.y
                    ));
                }
                result.destination = Destination::new(i as i32, result.height - 1);
            }
            _ => {
                return Err(format!(
                    "Unknown character {}, line {}, col {}",
                    char::from(c),
                    result.height - 1,
                    i
                ))
            }
        }
    }

    Ok(())
}


#[cfg(test)]
mod test_state {

    use super::*;

    #[test]
    fn build_fails_on_emptystring() {
        let s = State::build_from_str("");

        assert!(s.is_err())
    }

    #[test]
    fn build_correct_height() {
        let source = "\
            .\n\
            T\n\
            d\n\
            .\n\
            ";

        let res = State::build_from_str(source);
        assert_matches!( res, Ok( State { height: 4, .. } ))
    }

    #[test]
    fn build_correct_width() {
        let source = "\
        dT...\n\
        ";

        let res = State::build_from_str(source);
        assert_matches!( res, Ok( State { width: 5, .. } ))
    }

    #[test]
    fn build_fails_unequal_width() {
        let source = "\
        ...\n\
        ....\n\
        ";

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn build_fails_unknown_character() {
        let source = "\
        ....\n\
        ..<.\n\
        ";

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
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

        let mut expected_state = State::build_empty();
        expected_state.width = 5;
        expected_state.height = 5;
        expected_state.taxi = Taxi::new(1, 1);
        expected_state.passenger = Passenger::new(1, 1);
        expected_state.destination = Destination::new(0, 0);

        match State::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(res_state) => {
                assert_eq!(res_state, expected_state);
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

        let mut expected_state = State::build_empty();
        expected_state.width = 5;
        expected_state.height = 5;
        expected_state.walls = vec![ Wall::new(3,2) ];
        expected_state.taxi = Taxi::new(1, 1);
        expected_state.passenger = Passenger::new(1, 1);
        expected_state.destination = Destination::new(0, 0);

        match State::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(res_state) => {
                assert_eq!(res_state, expected_state);
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

        let mut expected_state = State::build_empty();
        expected_state.width = 10;
        expected_state.height = 7;
        expected_state.walls = vec![
                Wall::new(0,0),
                Wall::new(1,0),
                Wall::new(2,0),
                Wall::new(3,0),
                Wall::new(4,0),
                Wall::new(5,0),
                Wall::new(6,0),
                Wall::new(7,0),
                Wall::new(8,0),
                Wall::new(9,0),
                Wall::new(0,1),
                Wall::new(4,1),
                Wall::new(9,1),
                Wall::new(0,2),
                Wall::new(4,2),
                Wall::new(9,2),
                Wall::new(0,3),
                Wall::new(6,3),
                Wall::new(9,3),
                Wall::new(0,4),
                Wall::new(2,4),
                Wall::new(6,4),
                Wall::new(9,4),
                Wall::new(0,5),
                Wall::new(2,5),
                Wall::new(6,5),
                Wall::new(9,5),
                Wall::new(0,6),
                Wall::new(1,6),
                Wall::new(2,6),
                Wall::new(3,6),
                Wall::new(4,6),
                Wall::new(5,6),
                Wall::new(6,6),
                Wall::new(7,6),
                Wall::new(8,6),
                Wall::new(9,6),
            ];
        expected_state.taxi = Taxi::new(1, 1);
        expected_state.passenger = Passenger::new(1, 1);
        expected_state.destination = Destination::new(7, 4);

        match State::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(res_state) => {
                assert_eq!(res_state, expected_state);
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

        let mut expected_state = State::build_empty();
        expected_state.width = 5;
        expected_state.height = 5;
        expected_state.taxi = Taxi::new(1, 3);
        expected_state.passenger = Passenger::new(1, 3);
        expected_state.destination = Destination::new(0, 0);

        match State::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(res_state) => {
                assert_eq!(res_state, expected_state);
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

        let mut expected_state = State::build_empty();
        expected_state.width = 5;
        expected_state.height = 5;
        expected_state.taxi = Taxi::new(1, 3);
        expected_state.passenger = Passenger::new(3, 1);
        expected_state.destination = Destination::new(0, 0);

        match State::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(res_state) => {
                assert_eq!(res_state, expected_state);
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

        let mut expected_state = State::build_empty();
        expected_state.width = 5;
        expected_state.height = 5;
        expected_state.taxi = Taxi::new(1, 3);
        expected_state.passenger = Passenger::new(3, 1);
        expected_state.destination = Destination::new(3, 4);

        match State::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(res_state) => {
                assert_eq!(res_state, expected_state);
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

        let mut expected_state = State::build_empty();
        expected_state.width = 5;
        expected_state.height = 5;
        expected_state.taxi = Taxi::new(1, 3);
        expected_state.passenger = Passenger::new(1, 3);
        expected_state.destination = Destination::new(3, 4);

        match State::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(res_state) => {
                assert_eq!(res_state, expected_state);
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

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
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

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
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

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
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

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
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

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
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

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
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

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
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

        let res = State::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn output_matches_str_simple() {
        let source = "\
        d.\n\
        .T\n\
        ";

        match State::build_from_str(source) {
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

        match State::build_from_str(source) {
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
