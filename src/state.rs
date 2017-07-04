

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
}

impl State {
    pub fn build_from_str(source: &str) -> Result<State, String> {

        let mut lines = source.lines();

        match lines.next() {

            None => Err(String::from("Empty string pased to State::new")),

            Some(first_line) => {
                let width = first_line.len();

                let mut result = State {
                    width: width as i32,
                    height: 1,
                    walls: vec![],
                };

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

        let expected_state = State {
            width: 5,
            height: 5,
            walls: vec![],
        };


        match State::build_from_str(source) {
            Err(_) => assert!(false),
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

        let expected_state = State {
            width: 5,
            height: 5,
            walls: vec![ Wall::new(3,2) ],
        };

        match State::build_from_str(source) {
            Err(_) => assert!(false),
            Ok(res_state) => {
                assert_eq!(res_state, expected_state);
            }
        }
    }
}
