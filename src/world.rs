
use position::Position;

#[derive(Debug, PartialEq)]
pub struct Wall {
    pub position: Position,
}


impl Wall {
    pub fn new(x: i32, y: i32) -> Wall {
        Wall { position: Position::new(x, y) }
    }
}

#[derive(Debug, PartialEq)]
pub struct World {
    pub width: i32,
    pub height: i32,
    pub walls: Vec<Wall>,
}

impl World {
    fn build_empty() -> World {
        World {
            width: 0,
            height: 0,
            walls: vec![],
        }

    }

    pub fn build_from_str(source: &str) -> Result<World, String> {

        let mut lines = source.lines();

        match lines.next() {

            None => Err(String::from("Empty string passed to World::new")),

            Some(first_line) => {

                let mut result = World::build_empty();
                result.width = first_line.len() as i32;
                result.height = 1;

                parse_line(first_line, &mut result)?;

                for l in lines {

                    result.height += 1;

                    parse_line(l, &mut result)?;

                    let current_width = l.len();
                    if result.width != current_width as i32 {
                        return Err(format!(
                            "Failed to create world, line {} has width {} expected {}",
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



    pub fn display_bytes(&self) -> Vec<u8> {

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

        bytes
    }
}

fn parse_line(line: &str, result: &mut World) -> Result<(), String> {

    for (i, c) in line.bytes().enumerate() {
        match c {
            b'.' => (),
            b'#' => {
                let w = Wall::new(i as i32, result.height - 1);
                result.walls.push(w);
            }
            _ => (),
        }
    }

    Ok(())
}


#[cfg(test)]
mod test_state {

    use super::*;

    #[test]
    fn build_fails_on_emptystring() {
        let w = World::build_from_str("");

        assert!(w.is_err())
    }

    #[test]
    fn build_correct_height() {
        let source = "\
            .\n\
            T\n\
            d\n\
            .\n\
            ";

        let res = World::build_from_str(source);
        assert_matches!( res, Ok( World { height: 4, .. } ))
    }

    #[test]
    fn build_correct_width() {
        let source = "\
        dT...\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Ok( World { width: 5, .. } ))
    }

    #[test]
    fn build_fails_unequal_width() {
        let source = "\
        ...\n\
        ....\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn build_complex() {
        let source = "\
        ##########\n\
        #...#....#\n\
        #...#....#\n\
        #.....#..#\n\
        #.#...#..#\n\
        #.#...#..#\n\
        ##########\n\
        ";

        let mut expected_w = World::build_empty();
        expected_w.width = 10;
        expected_w.height = 7;
        expected_w.walls = vec![
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

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                assert_eq!(w, expected_w);
            }
        }
    }
}
