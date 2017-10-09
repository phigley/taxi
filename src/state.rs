

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
pub struct State<'a> {
    pub world: &'a World,
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

            Some(_) => {

                let mut result = State::build_empty(world);

                let mut current_y = 0;

                let mut process_line = true;

                for l in lines {

                    if process_line {
                        parse_line(l, current_y, &mut result)?;
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



    pub fn display(&self) -> String {

        let world_strings = self.world.display_strings();

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

    pub fn apply_action(&self, action: Actions) -> State<'a> {

        if !self.valid_action(action) {
            *self
        } else {
            let delta = position_delta(action);

            let new_taxi_pos = self.taxi.position + delta;

            if !self.valid_position(new_taxi_pos) {
                *self
            } else {
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
    }

    fn valid_action(&self, action: Actions) -> bool {
        match action {
            Actions::North => {
                let w = self.world.get_wall(&self.taxi.position);
                !w.north
            }

            Actions::South => {
                let w = self.world.get_wall(&self.taxi.position);
                !w.south
            }

            Actions::East => {
                let w = self.world.get_wall(&self.taxi.position);
                !w.east
            }

            Actions::West => {
                let w = self.world.get_wall(&self.taxi.position);
                !w.west
            }
        }
    }

    pub fn at_destination(&self) -> bool {
        self.passenger.position == self.destination.position
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
            // for w in self.world.walls.iter() {
            //     if w.position == position {
            //         return false;
            //     }
            // }

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

    for (i, c) in line.chars().enumerate() {
        if i % 2 == 1 {
            let x = (i / 2) as i32;

            if x >= result.world.width {
                return Err(format!(
                    "Reading character position {} which is greater than world's width of {}",
                    x,
                    result.world.width
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
                let mut expected_state = State::build_empty(&w);
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
                let mut expected_state = State::build_empty(&w);
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
                let mut expected_state = State::build_empty(&w);

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
                let mut expected_state = State::build_empty(&w);

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
                        let result = state.apply_action(Actions::North);
                        let result_str = result.display();
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
                        let result0 = state.apply_action(Actions::North);
                        let result0_str = result0.display();
                        assert_eq!(expected0, result0_str);

                        let result1 = result0.apply_action(Actions::West);
                        let result1_str = result1.display();
                        assert_eq!(expected1, result1_str);

                        assert_eq!(result1.passenger, Passenger::new(0,2));
                    }
                }
            }
        }
    }

}
