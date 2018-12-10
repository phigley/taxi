use std::fmt;
use std::iter;

use crate::actions::Actions;
use crate::position::Position;

#[derive(PartialEq, Clone, Debug)]
pub struct Wall {
    pub north: bool,
    pub south: bool,
    pub east: bool,
    pub west: bool,
}

impl Wall {
    fn new() -> Wall {
        Wall {
            north: false,
            south: false,
            east: false,
            west: false,
        }
    }
}

impl fmt::Display for Wall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
struct FixedPosition {
    id: char,
    position: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Costs {
    pub movement: f64,
    pub miss_pickup: f64,
    pub miss_dropoff: f64,
    pub empty_dropoff: f64,
}

impl Costs {
    pub fn new(movement: f64, miss_pickup: f64, miss_dropoff: f64, empty_dropoff: f64) -> Self {
        Costs {
            movement,
            miss_pickup,
            miss_dropoff,
            empty_dropoff,
        }
    }
}

impl Default for Costs {
    fn default() -> Self {
        Costs::new(-1.0, -10.0, -10.0, -10.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct World {
    pub width: i32,
    pub height: i32,
    walls: Vec<Vec<Wall>>,
    fixed_positions: Vec<FixedPosition>,

    pub costs: Costs,
}

#[derive(Debug, PartialEq)]
pub enum ActionAffect {
    Invalid,
    Move(Position),
    PickUp(char),
    DropOff(char),
}

pub enum Error {
    EmptyString,
    Parse { source: String, error: ParseError },
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::EmptyString => write!(f, "Attempted to build world from empty string."),
            Error::Parse {
                ref source,
                ref error,
            } => write!(f, "Parse failure: {:?}\nSource string:\n{}", error, source),
        }
    }
}

pub enum ParseError {
    LineTooLong {
        line: usize,
        num_chars: usize,
        expected_num_chars: usize,
    },
    DuplicateFixedPosition {
        id: char,
    },
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ParseError::LineTooLong {
                line,
                num_chars,
                expected_num_chars,
            } => write!(
                f,
                "Line {} has width {} which does not match initial width of {}.",
                line, num_chars, expected_num_chars
            ),

            ParseError::DuplicateFixedPosition { id } => {
                write!(f, "Found duplicate fixed position '{}'.", id)
            }
        }
    }
}

impl World {
    pub fn build_from_str(source: &str, costs: Costs) -> Result<World, Error> {
        let mut lines = source.lines();

        if let Some(first_line) = lines.next() {
            let line_width = first_line.chars().count();

            let mut line_count = 1;

            let width = line_width / 2;

            let mut fixed_positions = Vec::new();

            let mut walls = Vec::new();

            let mut wall_row = Vec::with_capacity(width);
            wall_row.extend(iter::repeat(Wall::new()).take(width));
            parse_wall_line(first_line, line_count, width, None, Some(&mut wall_row)).map_err(
                |error| Error::Parse {
                    source: String::from(source),
                    error,
                },
            )?;

            while let (Some(content_line), Some(wall_line)) = (lines.next(), lines.next()) {
                let mut next_wall_row = Vec::with_capacity(width);
                next_wall_row.extend(iter::repeat(Wall::new()).take(width));

                line_count += 1;
                parse_content_line(
                    content_line,
                    line_count,
                    width,
                    &mut wall_row,
                    &mut fixed_positions,
                )
                .map_err(|error| Error::Parse {
                    source: String::from(source),
                    error,
                })?;

                line_count += 1;
                parse_wall_line(
                    wall_line,
                    line_count,
                    width,
                    Some(&mut wall_row),
                    Some(&mut next_wall_row),
                )
                .map_err(|error| Error::Parse {
                    source: String::from(source),
                    error,
                })?;

                walls.push(wall_row);
                wall_row = next_wall_row;
            }

            let height = walls.len() as i32;

            Ok(World {
                width: width as i32,
                height: height as i32,
                walls,
                fixed_positions,

                costs,
            })
        } else {
            Err(Error::EmptyString)
        }
    }

    pub fn get_fixed_position(&self, id: char) -> Option<&Position> {
        for fp in &self.fixed_positions {
            if fp.id == id {
                return Some(&fp.position);
            }
        }

        None
    }

    pub fn get_fixed_index(&self, id: char) -> Option<usize> {
        for (i, fp) in self.fixed_positions.iter().enumerate() {
            if fp.id == id {
                return Some(i);
            }
        }

        None
    }

    pub fn get_fixed_id(&self, position: &Position) -> Option<char> {
        for fp in &self.fixed_positions {
            if fp.position == *position {
                return Some(fp.id);
            }
        }

        None
    }

    pub fn max_reward(&self) -> f64 {
        0.0
    }

    pub fn num_fixed_positions(&self) -> usize {
        self.fixed_positions.len()
    }

    pub fn get_fixed_id_from_index(&self, index: usize) -> Option<char> {
        if index < self.fixed_positions.len() {
            Some(self.fixed_positions[index].id)
        } else {
            None
        }
    }

    pub fn get_wall(&self, position: &Position) -> &Wall {
        &self.walls[position.y as usize][position.x as usize]
    }

    pub fn determine_affect(&self, position: &Position, action: Actions) -> ActionAffect {
        match action {
            Actions::North => {
                if position.y > 0 && !self.get_wall(position).north {
                    ActionAffect::Move(Position::new(0, -1))
                } else {
                    ActionAffect::Invalid
                }
            }

            Actions::South => {
                if position.y < (self.height - 1) && !self.get_wall(position).south {
                    ActionAffect::Move(Position::new(0, 1))
                } else {
                    ActionAffect::Invalid
                }
            }

            Actions::East => {
                if position.x < (self.width - 1) && !self.get_wall(position).east {
                    ActionAffect::Move(Position::new(1, 0))
                } else {
                    ActionAffect::Invalid
                }
            }

            Actions::West => {
                if position.x > 0 && !self.get_wall(position).west {
                    ActionAffect::Move(Position::new(-1, 0))
                } else {
                    ActionAffect::Invalid
                }
            }

            Actions::PickUp => {
                if let Some(id) = self.get_fixed_id(position) {
                    ActionAffect::PickUp(id)
                } else {
                    ActionAffect::Invalid
                }
            }

            Actions::DropOff => {
                if let Some(id) = self.get_fixed_id(position) {
                    ActionAffect::DropOff(id)
                } else {
                    ActionAffect::Invalid
                }
            }
        }
    }

    pub fn display(&self) -> String {
        let mut result = String::new();

        for s in self.display_strings() {
            result += &s;
            result.push('\n');
        }

        result
    }

    pub fn display_strings(&self) -> Vec<String> {
        let line_count = (2 * self.height + 1) as usize;
        let mut result = Vec::with_capacity(line_count);

        let mut previous_row: Option<&[Wall]> = None;

        for (y, row) in self.walls.iter().enumerate() {
            let mut upper_chars = String::new();
            let mut chars = String::new();

            let mut previous_wall = None;

            for (x, w) in row.iter().enumerate() {
                let upper_wall = if let Some(previous_row) = previous_row {
                    Some(&previous_row[x])
                } else {
                    None
                };

                upper_chars.push(calc_upper_left_char(w, previous_wall, upper_wall));
                upper_chars.push(if w.north { '─' } else { ' ' });

                chars.push(if w.west { '│' } else { ' ' });

                let mut content_char = '.';

                for fp in &self.fixed_positions {
                    if fp.position.y == (y as i32) && fp.position.x == (x as i32) {
                        content_char = fp.id;
                        break;
                    }
                }

                chars.push(content_char);

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

        (true, false, false, false)
        | (false, false, true, false)
        | (false, true, false, false)
        | (false, false, false, true)
        | (false, false, false, false) => ' ',
    }
}

fn parse_wall_line(
    line: &str,
    line_count: usize,
    width: usize,
    mut previous_row: Option<&mut [Wall]>,
    mut row: Option<&mut [Wall]>,
) -> Result<(), ParseError> {
    let mut num_chars_read = 0;
    let expected_num_chars = 2 * width + 1;

    for (i, c) in line.chars().enumerate() {
        num_chars_read += 1;

        if num_chars_read > expected_num_chars {
            break;
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
        Err(ParseError::LineTooLong {
            line: line_count,
            num_chars: line.chars().count(),
            expected_num_chars,
        })
    }
}

fn parse_content_line(
    line: &str,
    line_count: usize,
    width: usize,
    wall_row: &mut Vec<Wall>,
    fixed_positions: &mut Vec<FixedPosition>,
) -> Result<(), ParseError> {
    let mut num_chars_read = 0;
    let expected_num_chars = 2 * width + 1;

    let y = (line_count - 1) / 2;

    for (i, c) in line.chars().enumerate() {
        num_chars_read += 1;

        if num_chars_read > expected_num_chars {
            break;
        }

        let x = i / 2;

        if i % 2 == 1 {
            // odd characters are points themselves
            if c != '.' {
                // for now, ignore the taxi, passenger, and destination characters.
                if c != 't' && c != 'T' && c != 'd' && c != 'D' && c != 'p' {
                    for fp in fixed_positions.iter() {
                        if fp.id == c {
                            return Err(ParseError::DuplicateFixedPosition { id: c });
                        }
                    }
                }

                fixed_positions.push(FixedPosition {
                    id: c,
                    position: Position::new(x as i32, y as i32),
                })
            }
        } else if c == '│' {
            // even characters can only be walls
            if x < width {
                wall_row[x].west = true;
            }

            if x > 0 {
                wall_row[x - 1].east = true;
            }
        }
    }

    if num_chars_read == expected_num_chars {
        Ok(())
    } else {
        Err(ParseError::LineTooLong {
            line: line_count,
            num_chars: line.chars().count(),
            expected_num_chars,
        })
    }
}

#[cfg(test)]
mod test_world {

    use super::*;

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

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Ok(World { height: 4, .. }))
    }

    #[test]
    fn build_correct_width() {
        let source = "\
                      ┌─────────┐\n\
                      │d T . . .│\n\
                      └─────────┘\n\
                      ";

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Ok(World { width: 5, .. }))
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

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Err(_))
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

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Err(_))
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

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Err(_))
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

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Err(_))
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

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Err(_))
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

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Err(_))
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

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Err(_))
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

        let res = World::build_from_str(source, Costs::default());
        assert_matches!(res, Err(_))
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
            fixed_positions: vec![],

            costs: Costs::default(),
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
        expected_w.walls = vec![vec![build_wall("nsew")]];

        match World::build_from_str(source, Costs::default()) {
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
        expected_w.walls = vec![
            vec![build_wall("nw"), build_wall("n"), build_wall("ne")],
            vec![build_wall("w"), build_wall(""), build_wall("e")],
            vec![build_wall("w"), build_wall(""), build_wall("e")],
            vec![build_wall("sw"), build_wall("s"), build_wall("se")],
        ];

        match World::build_from_str(source, Costs::default()) {
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
        expected_w.walls = vec![
            vec![
                build_wall("nw"),
                build_wall("ns"),
                build_wall("n"),
                build_wall("ne"),
            ],
            vec![
                build_wall("ew"),
                build_wall("nw"),
                build_wall(""),
                build_wall("e"),
            ],
            vec![
                build_wall("sew"),
                build_wall("sw"),
                build_wall("s"),
                build_wall("se"),
            ],
        ];

        match World::build_from_str(source, Costs::default()) {
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
            vec![
                build_wall("nw"),
                build_wall("ne"),
                build_wall("nw"),
                build_wall("n"),
                build_wall("ne"),
            ],
            vec![
                build_wall("w"),
                build_wall("e"),
                build_wall("w"),
                build_wall(""),
                build_wall("e"),
            ],
            vec![
                build_wall("w"),
                build_wall(""),
                build_wall(""),
                build_wall(""),
                build_wall("e"),
            ],
            vec![
                build_wall("we"),
                build_wall("w"),
                build_wall("e"),
                build_wall("w"),
                build_wall("e"),
            ],
            vec![
                build_wall("swe"),
                build_wall("sw"),
                build_wall("se"),
                build_wall("sw"),
                build_wall("se"),
            ],
        ];

        match World::build_from_str(source, Costs::default()) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                assert_eq!(w, expected_w);
            }
        }
    }

    #[test]
    fn build_simple_fixed_positions() {
        let source = "\
                      ┌─────┐\n\
                      │A . .│\n\
                      │     │\n\
                      │. B .│\n\
                      │     │\n\
                      │. . C│\n\
                      │     │\n\
                      │. . .│\n\
                      └─────┘\n\
                      ";

        let mut expected_w = build_empty_world();
        expected_w.width = 3;
        expected_w.height = 4;
        expected_w.walls = vec![
            vec![build_wall("nw"), build_wall("n"), build_wall("ne")],
            vec![build_wall("w"), build_wall(""), build_wall("e")],
            vec![build_wall("w"), build_wall(""), build_wall("e")],
            vec![build_wall("sw"), build_wall("s"), build_wall("se")],
        ];
        expected_w.fixed_positions = vec![
            FixedPosition {
                id: 'A',
                position: Position::new(0, 0),
            },
            FixedPosition {
                id: 'B',
                position: Position::new(1, 1),
            },
            FixedPosition {
                id: 'C',
                position: Position::new(2, 2),
            },
        ];

        match World::build_from_str(source, Costs::default()) {
            Err(msg) => panic!(msg),
            Ok(w) => {
                assert_eq!(w, expected_w);
            }
        }
    }
}
