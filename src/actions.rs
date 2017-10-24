
use std::fmt;
use rand::{Rand, Rng};

#[cfg(test)]
use distribution::MeasureDistribution;

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
    pub const NUM_ELEMENTS: usize = 4;

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

impl Rand for Actions {
    fn rand<R: Rng>(rng: &mut R) -> Actions {

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

    use std::f64;
    use rand;
    use rand::Rng;

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
        let expected_std_dev = 1.0 / (max_iterations as f64).sqrt();

        let mut north_dist = MeasureDistribution::new();
        let mut south_dist = MeasureDistribution::new();
        let mut east_dist = MeasureDistribution::new();
        let mut west_dist = MeasureDistribution::new();
        let mut pickup_dist = MeasureDistribution::new();
        let mut dropoff_dist = MeasureDistribution::new();

        let mut rng = rand::thread_rng();
        for _ in 0..max_iterations {

            let action: Actions = rng.gen();

            north_dist.add_value(if action == Actions::North { 1.0 } else { 0.0 });
            south_dist.add_value(if action == Actions::South { 1.0 } else { 0.0 });
            east_dist.add_value(if action == Actions::East { 1.0 } else { 0.0 });
            west_dist.add_value(if action == Actions::West { 1.0 } else { 0.0 });
            pickup_dist.add_value(if action == Actions::PickUp { 1.0 } else { 0.0 });
            dropoff_dist.add_value(if action == Actions::DropOff { 1.0 } else { 0.0 });
        }

        let max_avg_dev = 4.0 * expected_std_dev;
        let expected_avg = 1.0 / 6.0;

        println!(
            "expected avg = {} expected std dev = {} max_dev = {}",
            expected_avg,
            expected_std_dev,
            max_avg_dev
        );


        let (north_avg, north_dev) = north_dist.get_distribution();
        println!(
            "north = {:?} delta = {}",
            (north_avg, north_dev),
            north_avg - expected_avg
        );

        let (south_avg, south_dev) = south_dist.get_distribution();
        println!(
            "south = {:?} delta = {}",
            (south_avg, south_dev),
            south_avg - expected_avg
        );

        let (east_avg, east_dev) = east_dist.get_distribution();
        println!(
            "east = {:?} delta = {}",
            (east_avg, east_dev),
            east_avg - expected_avg
        );

        let (west_avg, west_dev) = west_dist.get_distribution();
        println!(
            "west = {:?} delta = {}",
            (west_avg, west_dev),
            west_avg - expected_avg
        );

        let (pickup_avg, pickup_dev) = pickup_dist.get_distribution();
        println!(
            "pickup = {:?} delta = {}",
            (pickup_avg, pickup_dev),
            pickup_avg - expected_avg
        );

        let (dropoff_avg, dropoff_dev) = dropoff_dist.get_distribution();
        println!(
            "dropoff = {:?} delta = {}",
            (dropoff_avg, dropoff_dev),
            dropoff_avg - expected_avg
        );

        assert!(north_avg - expected_avg < expected_avg * max_avg_dev);
        assert!(north_avg - expected_avg > -expected_avg * max_avg_dev);
        assert!(north_dev < expected_std_dev * 1.1);

        assert!(south_avg - expected_avg < expected_avg * max_avg_dev);
        assert!(south_avg - expected_avg > -expected_avg * max_avg_dev);
        assert!(south_dev < expected_std_dev * 1.1);

        assert!(east_avg - expected_avg < expected_avg * max_avg_dev);
        assert!(east_avg - expected_avg > -expected_avg * max_avg_dev);
        assert!(east_dev < expected_std_dev * 1.1);

        assert!(west_avg - expected_avg < expected_avg * max_avg_dev);
        assert!(west_avg - expected_avg > -expected_avg * max_avg_dev);
        assert!(west_dev < expected_std_dev * 1.1);

        assert!(pickup_avg - expected_avg < expected_avg * max_avg_dev);
        assert!(pickup_avg - expected_avg > -expected_avg * max_avg_dev);
        assert!(pickup_dev < expected_std_dev * 1.1);

        assert!(dropoff_avg - expected_avg < expected_avg * max_avg_dev);
        assert!(dropoff_avg - expected_avg > -expected_avg * max_avg_dev);
        assert!(dropoff_dev < expected_std_dev * 1.1);
    }
}
