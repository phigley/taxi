

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

                Ok(result)


            }
        }
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
                result.taxi = Taxi::new(i as i32, result.height - 1);
            }
            b'p' => {
                result.passenger = Passenger::new(i as i32, result.height - 1);
            }
            b'd' => {
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
            .\n\
            .\n\
            .\n\
            ";

        let res = State::build_from_str(source);
        assert_matches!( res, Ok( State { height: 4, .. } ))
    }

    #[test]
    fn build_correct_width() {
        let source = "\
        .....\n\
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
        .....\n\
        .....\n\
        .....\n\
        .....\n\
        .....\n\
        ";

        let mut expected_state = State::build_empty();
        expected_state.width = 5;
        expected_state.height = 5;

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
        .....\n\
        .....\n\
        ...#.\n\
        .....\n\
        .....\n\
        ";

        let mut expected_state = State::build_empty();
        expected_state.width = 5;
        expected_state.height = 5;
        expected_state.walls = vec![ Wall::new(3,2) ];

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
        #...#....#\n\
        #...#....#\n\
        #.....#..#\n\
        #.#...#..#\n\
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
        .....\n\
        .....\n\
        .....\n\
        .t...\n\
        .....\n\
        ";

        let mut expected_state = State::build_empty();
        expected_state.width = 5;
        expected_state.height = 5;
        expected_state.taxi = Taxi::new(1, 3);

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
        .....\n\
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
}
