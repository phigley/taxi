use rand::Rng;
use rand::distributions::{ Distribution, Standard};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Actions {
    North,
    South,
    East,
    West,
    PickUp,
    DropOff,
}

impl Actions {
    pub const NUM_ELEMENTS: usize = 6;

    pub fn to_index(&self) -> usize {
        match *self {
            Actions::North => 0,
            Actions::South => 1,
            Actions::East => 2,
            Actions::West => 3,
            Actions::PickUp => 4,
            Actions::DropOff => 5,
        }
    }

    pub fn from_index(i: usize) -> Option<Actions> {
        match i {
            0 => Some(Actions::North),
            1 => Some(Actions::South),
            2 => Some(Actions::East),
            3 => Some(Actions::West),
            4 => Some(Actions::PickUp),
            5 => Some(Actions::DropOff),
            _ => None,
        }
    }
}
impl Distribution<Actions> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Actions {
        let actions = [
            Actions::North,
            Actions::South,
            Actions::East,
            Actions::West,
            Actions::PickUp,
            Actions::DropOff,
        ];

        actions[rng.gen_range(0, 6)]
    }
}

impl fmt::Display for Actions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Actions::North => write!(f, "N"),
            Actions::South => write!(f, "S"),
            Actions::East => write!(f, "E"),
            Actions::West => write!(f, "W"),
            Actions::PickUp => write!(f, "P"),
            Actions::DropOff => write!(f, "D"),
        }
    }
}

#[cfg(test)]
mod test_actions {

    use super::*;

    use rand;
    use rand::Rng;
    use std::f64;

    fn find_action(desired_action: Actions, max_iterations: u32) -> bool {
        let mut rng = rand::thread_rng();

        let mut iterations_remaining = max_iterations;
        loop {
            if iterations_remaining == 0 {
                break false;
            }

            iterations_remaining -= 1;

            let action: Actions = rng.gen();

            if action == desired_action {
                break true;
            }
        }
    }

    #[test]
    fn random_action_north() {
        let found_action = find_action(Actions::North, 500);
        assert!(found_action);
    }

    #[test]
    fn random_action_south() {
        let found_action = find_action(Actions::South, 500);
        assert!(found_action);
    }

    #[test]
    fn random_action_east() {
        let found_action = find_action(Actions::East, 500);
        assert!(found_action);
    }

    #[test]
    fn random_action_west() {
        let found_action = find_action(Actions::West, 500);
        assert!(found_action);
    }

    #[test]
    fn random_action_pickup() {
        let found_action = find_action(Actions::PickUp, 500);
        assert!(found_action);
    }

    #[test]
    fn random_action_dropoff() {
        let found_action = find_action(Actions::DropOff, 500);
        assert!(found_action);
    }

    #[test]
    fn distribution() {
        let max_iterations = 1000000;

        let mut counts = vec![0.0f64; Actions::NUM_ELEMENTS];

        let mut rng = rand::thread_rng();
        for _ in 0..max_iterations {
            let action: Actions = rng.gen();

            counts[action.to_index()] += 1.0;
        }

        // chi-squared should not exceed this for 95% confidence.
        let p_05 = 11.07;

        let expected_count = (max_iterations as f64) / (counts.len() as f64);

        let mut chi_sqr = 0.0f64;
        for count in &counts {
            let delta = count - expected_count;
            chi_sqr += (delta * delta) / expected_count;
        }

        println!("");
        println!(
            "north count = {}, ratio = {}",
            counts[Actions::North.to_index()],
            counts[Actions::North.to_index()] / expected_count
        );

        println!(
            "south count = {}, ratio = {}",
            counts[Actions::South.to_index()],
            counts[Actions::South.to_index()] / expected_count
        );

        println!(
            "east count = {}, ratio = {}",
            counts[Actions::East.to_index()],
            counts[Actions::East.to_index()] / expected_count
        );

        println!(
            "west count = {}, ratio = {}",
            counts[Actions::West.to_index()],
            counts[Actions::West.to_index()] / expected_count
        );

        println!(
            "pickup count = {}, ratio = {}",
            counts[Actions::PickUp.to_index()],
            counts[Actions::PickUp.to_index()] / expected_count
        );

        println!(
            "dropoff count = {}, ratio = {}",
            counts[Actions::DropOff.to_index()],
            counts[Actions::DropOff.to_index()] / expected_count
        );

        println!("chi-squared = {}, 95% confidence = {}", chi_sqr, p_05);

        assert!(chi_sqr < p_05);
    }
}
