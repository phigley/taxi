use std::ops;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position { x: x, y: y }
    }
}

impl ops::Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Position {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[cfg(test)]
mod test_position {

    use super::*;

    #[test]
    fn create() {
        let a = Position::new(3, 4);

        assert_eq!(a, Position { x: 3, y: 4 })
    }

    #[test]
    fn add() {
        let a = Position::new(1, 2);
        let b = Position::new(3, 4);

        assert_eq!(a + b, Position { x: 4, y: 6 })
    }
}
