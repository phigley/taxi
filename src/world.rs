
use std::iter;
use std::fmt;

use position::Position;
use actions::Actions;

#[derive(PartialEq, Clone)]
pub struct Wall {
    pub north: bool,
    pub south: bool,
    pub east: bool,
    pub west: bool,
}


impl Wall {
    pub fn new() -> Wall {
        Wall {
            north: false,
            south: false,
            east: false,
            west: false,
        }
    }
}

impl fmt::Debug for Wall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wall('")?;

        if self.north {
            write!(f, "n")?;
        }

        if self.south {
            write!(f, "s")?;
        }

        if self.east {
            write!(f, "e")?;
        }

        if self.west {
            write!(f, "w")?;
        }

        write!(f, "')")
    }
}

#[derive(Debug, PartialEq)]
pub struct World {
    pub width: i32,
    pub height: i32,
    walls: Vec<Vec<Wall>>,
}

impl World {
    pub fn build_from_str(source: &str) -> Result<World, String> {

        let mut lines = source.lines();

        if let Some(first_line) = lines.next() {

            let line_width = first_line.chars().count();

            let mut line_count = 1;

            let width = line_width / 2;

            let mut walls = Vec::new();

            let mut wall_row = Vec::with_capacity(width);
            wall_row.extend(iter::repeat(Wall::new()).take(width));
            parse_wall_line(first_line, line_count, width, None, Some(&mut wall_row))?;

            while let (Some(content_line), Some(wall_line)) = (lines.next(), lines.next()) {

                let mut next_wall_row = Vec::with_capacity(width);
                next_wall_row.extend(iter::repeat(Wall::new()).take(width));

                parse_content_line(content_line, line_count, width, &mut wall_row)?;
                line_count += 1;
                parse_wall_line(
                    &wall_line,
                    line_count,
                    width,
                    Some(&mut wall_row),
                    Some(&mut next_wall_row),
                )?;

                walls.push(wall_row);
                wall_row = next_wall_row;
            }

            let height = walls.len() as i32;

            Ok(World {
                width: width as i32,
                height: height as i32,
                walls,
            })

        } else {
            Err(String::from("Empty string passed to World::new"))
        }
    }

    fn get_wall(&self, positon: &Position) -> &Wall {
        &self.walls[positon.y as usize][positon.x as usize]
    }

    pub fn valid_action(&self, position: &Position, action: Actions) -> bool {
        match action {
            Actions::North => position.y > 0 && !self.get_wall(&position).north,
            Actions::South => position.y < (self.height - 1) && !self.get_wall(&position).south,
            Actions::East => position.x < (self.width - 1) && !self.get_wall(&position).east,
            Actions::West => position.x > 0 && !self.get_wall(&position).west,
        }
    }

    pub fn display_strings(&self) -> Vec<String> {

        let line_count = (2 * self.height + 1) as usize;
        let mut result = Vec::with_capacity(line_count);

        let mut previous_row: Option<&[Wall]> = None;

        for row in &self.walls {

            let mut upper_chars = String::new();
            let mut chars = String::new();

            let mut previous_wall = None;

            for (i, w) in row.iter().enumerate() {

                let upper_wall = if let Some(previous_row) = previous_row {
                    Some(&previous_row[i])
                } else {
                    None
                };

                upper_chars.push(calc_upper_left_char(w, previous_wall, upper_wall));
                upper_chars.push(if w.north { '─' } else { ' ' });

                chars.push(if w.west { '│' } else { ' ' });
                chars.push('.');

                previous_wall = Some(w);
            }

            if let Some(w) = previous_wall {

                let upper_wall = if let Some(previous_row) = previous_row {
                    Some(&previous_row[(self.width - 1) as usize])
                } else {
                    None
                };


                upper_chars.push(calc_upper_right_char(w, upper_wall));
                chars.push(if w.east { '│' } else { ' ' });
            }

            result.push(upper_chars);
            result.push(chars);

            previous_row = Some(row);
        }

        if let Some(r) = previous_row {

            let mut bottom_chars = String::new();
            let mut previous_wall = None;
            for w in r {

                bottom_chars.push(calc_lower_left_char(w, previous_wall));
                bottom_chars.push(if w.south { '─' } else { ' ' });

                previous_wall = Some(w);
            }

            if let Some(w) = previous_wall {
                bottom_chars.push(render_connection(w.east, false, false, w.south));
            }

            result.push(bottom_chars);
        }

        result
    }
}

fn calc_upper_left_char(
    current_wall: &Wall,
    previous_wall: Option<&Wall>,
    upper_wall: Option<&Wall>,
) -> char {

    let mut connect_north = false;
    let mut connect_south = false;
    let mut connect_east = false;
    let mut connect_west = false;


    if let Some(upper_wall) = upper_wall {
        if upper_wall.west {
            connect_north = true;
        }
    }

    if current_wall.west {
        connect_south = true;
    }

    if current_wall.north {
        connect_east = true;
    }


    if let Some(previous_wall) = previous_wall {
        if previous_wall.north {
            connect_west = true;
        }
    }

    render_connection(connect_north, connect_south, connect_east, connect_west)
}

fn calc_upper_right_char(current_wall: &Wall, upper_wall: Option<&Wall>) -> char {

    let mut connect_north = false;
    let mut connect_south = false;
    let connect_east = false;
    let mut connect_west = false;


    if let Some(upper_wall) = upper_wall {
        if upper_wall.east {
            connect_north = true;
        }
    }

    if current_wall.east {
        connect_south = true;
    }

    if current_wall.north {
        connect_west = true;
    }

    render_connection(connect_north, connect_south, connect_east, connect_west)
}

fn calc_lower_left_char(current_wall: &Wall, previous_wall: Option<&Wall>) -> char {

    let mut connect_north = false;
    let connect_south = false;
    let mut connect_east = false;
    let mut connect_west = false;

    if current_wall.west {
        connect_north = true;
    }

    if current_wall.south {
        connect_east = true;
    }

    if let Some(previous_wall) = previous_wall {
        if previous_wall.south {
            connect_west = true;
        }
    }

    render_connection(connect_north, connect_south, connect_east, connect_west)
}

fn render_connection(
    connect_north: bool,
    connect_south: bool,
    connect_east: bool,
    connect_west: bool,
) -> char {

    // ─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼

    match (connect_north, connect_south, connect_east, connect_west) {

        (true, true, true, true) => '┼',

        (true, true, true, false) => '├',
        (true, true, false, true) => '┤',
        (true, false, true, true) => '┴',
        (false, true, true, true) => '┬',

        (true, true, false, false) => '│',
        (true, false, true, false) => '└',
        (true, false, false, true) => '┘',
        (false, true, true, false) => '┌',
        (false, true, false, true) => '┐',
        (false, false, true, true) => '─',

        (true, false, false, false) => ' ',
        (false, true, false, false) => ' ',
        (false, false, true, false) => ' ',
        (false, false, false, true) => ' ',
        (false, false, false, false) => ' ',
    }

}

fn parse_wall_line(
    line: &str,
    line_count: usize,
    width: usize,
    mut previous_row: Option<&mut [Wall]>,
    mut row: Option<&mut [Wall]>,
) -> Result<(), String> {

    let mut num_chars_read = 0;
    let expected_num_chars = 2 * width + 1;

    for (i, c) in line.chars().enumerate() {

        num_chars_read += 1;

        if num_chars_read > expected_num_chars {
            return Err(format!(
                "Line {} longer than initial width of {}.",
                line_count,
                expected_num_chars
            ));
        }

        if i % 2 == 1 {

            let x = i / 2;

            if x > width {}



            if c == '─' {
                if let Some(ref mut prev) = previous_row {
                    (*prev)[x].south = true;
                }

                if let Some(ref mut current) = row {
                    current[x].north = true;
                }
            }
        }
    }

    if num_chars_read == expected_num_chars {
        Ok(())
    } else {
        Err(format!(
            "Failed to create world, line {} has width {} expected {}",
            line_count,
            num_chars_read,
            expected_num_chars,
        ))
    }

}

fn parse_content_line(
    line: &str,
    line_count: usize,
    width: usize,
    wall_row: &mut Vec<Wall>,
) -> Result<(), String> {

    let mut num_chars_read = 0;
    let expected_num_chars = 2 * width + 1;


    for (i, c) in line.chars().enumerate() {

        num_chars_read += 1;

        if num_chars_read > expected_num_chars {
            return Err(format!(
                "Line {} longer than initial width of {}.",
                line_count,
                expected_num_chars
            ));
        }

        if i % 2 == 0 {


            let x = i / 2;

            if c == '│' {
                if x < width {
                    wall_row[x].west = true;
                }

                if x > 0 {
                    wall_row[x - 1].east = true;
                }
            }
        }

    }

    if num_chars_read == expected_num_chars {
        Ok(())
    } else {
        Err(format!(
            "Failed to create world, line {} has width {} expected {}",
            line_count,
            num_chars_read,
            expected_num_chars,
        ))
    }
}


#[cfg(test)]
mod test_world {

    use super::*;

    #[test]
    fn build_fails_on_emptystring() {
        let w = World::build_from_str("");

        assert!(w.is_err())
    }

    #[test]
    fn build_correct_height() {
        let source = "\
            ┌─┐\n\
            │.│\n\
            │ │\n\
            │T│\n\
            │ │\n\
            │d│\n\
            │ │\n\
            │.│\n\
            └─┘";

        let res = World::build_from_str(source);
        assert_matches!( res, Ok( World { height: 4, .. } ))
    }

    #[test]
    fn build_correct_width() {
        let source = "\
        ┌─────────┐\n\
        │d T . . .│\n\
        └─────────┘\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Ok( World { width: 5, .. } ))
    }

    #[test]
    fn build_fails_short_content() {
        let source = "\
        ┌───────┐\n\
        │. . .│\n\
        │       │\n\
        │. . . .│\n\
        └───────┘\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn build_fails_short_wall() {
        let source = "\
        ┌─────────┐\n\
        │ . . . . │\n\
        │        │\n\
        │ . . . . │\n\
        └─────────┘\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn build_fails_short_initial() {
        let source = "\
        ┌────────┐\n\
        │ . . . . │\n\
        │         │\n\
        │ . . . . │\n\
        └─────────┘\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn build_fails_short_final() {
        let source = "\
        ┌─────────┐\n\
        │ . . . . │\n\
        │         │\n\
        │ . . . . │\n\
        └────────┘\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn build_fails_long_content() {
        let source = "\
        ┌───────┐\n\
        │. . . . │\n\
        │       │\n\
        │. . . .│\n\
        └───────┘\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn build_fails_long_wall() {
        let source = "\
        ┌─────────┐\n\
        │ . . . . │\n\
        │          │\n\
        │ . . . . │\n\
        └─────────┘\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn build_fails_long_initial() {
        let source = "\
        ┌──────────┐\n\
        │ . . . . │\n\
        │         │\n\
        │ . . . . │\n\
        └─────────┘\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    #[test]
    fn build_fails_long_final() {
        let source = "\
        ┌─────────┐\n\
        │ . . . . │\n\
        │         │\n\
        │ . . . . │\n\
        └──────────┘\n\
        ";

        let res = World::build_from_str(source);
        assert_matches!( res, Err( _ ))
    }

    fn build_wall(desc: &str) -> Wall {
        let mut result = Wall::new();

        for c in desc.chars() {
            match c {
                'n' => result.north = true,
                's' => result.south = true,
                'e' => result.east = true,
                'w' => result.west = true,
                _ => (),
            }
        }

        result
    }

    fn build_empty_world() -> World {
        World {
            width: 0,
            height: 0,
            walls: vec![],
        }

    }


    #[test]
    fn build_very_simple() {
        let source = "\
        ┌─┐\n\
        │.│\n\
        └─┘\n\
        ";

        let mut expected_w = build_empty_world();
        expected_w.width = 1;
        expected_w.height = 1;
        expected_w.walls = vec![ vec![
            build_wall("nsew")
        ] ];

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                assert_eq!(w, expected_w);
            }
        }
    }

    #[test]
    fn build_simple() {
        let source = "\
        ┌─────┐\n\
        │. . .│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. . .│\n\
        └─────┘\n\
        ";

        let mut expected_w = build_empty_world();
        expected_w.width = 3;
        expected_w.height = 4;
        expected_w.walls = vec![ vec![
            build_wall("nw"),
            build_wall("n"),
            build_wall("ne")
        ],
        vec![
            build_wall("w"),
            build_wall(""),
            build_wall("e")
        ],
        vec![
            build_wall("w"),
            build_wall(""),
            build_wall("e")
        ],
        vec![
            build_wall("sw"),
            build_wall("s"),
            build_wall("se")
        ] ];

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                assert_eq!(w, expected_w);
            }
        }
    }

    #[test]
    fn build_middle_wall() {
        let source = "\
        ┌───────┐\n\
        │. . . .│\n\
        │ ┌─    │\n\
        │.│. . .│\n\
        │ │     │\n\
        │.│. . .│\n\
        └─┴─────┘\n\
        ";

        let mut expected_w = build_empty_world();
        expected_w.width = 4;
        expected_w.height = 3;
        expected_w.walls = vec![ vec![
            build_wall("nw"),
            build_wall("ns"),
            build_wall("n"),
            build_wall("ne")
        ],
        vec![
            build_wall("ew"),
            build_wall("nw"),
            build_wall(""),
            build_wall("e")
        ],
        vec![
            build_wall("sew"),
            build_wall("sw"),
            build_wall("s"),
            build_wall("se")
        ] ];

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                assert_eq!(w, expected_w);
            }
        }
    }

    #[test]
    fn build_complex() {
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

        let mut expected_w = build_empty_world();
        expected_w.width = 5;
        expected_w.height = 5;
        expected_w.walls = vec![
            vec![ build_wall("nw"),
            build_wall("ne"),
            build_wall("nw"),
            build_wall("n"),
            build_wall("ne") ],

            vec![ build_wall("w"),
            build_wall("e"),
            build_wall("w"),
            build_wall(""),
            build_wall("e") ],

            vec![ build_wall("w"),
            build_wall(""),
            build_wall(""),
            build_wall(""),
            build_wall("e") ],

            vec![ build_wall("we"),
            build_wall("w"),
            build_wall("e"),
            build_wall("w"),
            build_wall("e") ],

            vec![ build_wall("swe"),
            build_wall("sw"),
            build_wall("se"),
            build_wall("sw"),
            build_wall("se") ],
        ];

        match World::build_from_str(source) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                assert_eq!(w, expected_w);
            }
        }
    }
}
