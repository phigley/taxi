
use rand::{Rand, Rng};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Actions {
    North,
    South,
    East,
    West,
}

impl Actions {
    pub const NUM_ELEMENTS: usize = 4;

    pub fn to_index(&self) -> usize {
        match *self {
            Actions::North => 0,
            Actions::South => 1,
            Actions::East => 2,
            Actions::West => 3,
        }
    }

    pub fn from_index(i: usize) -> Option<Actions> {
        match i {
            0 => Some(Actions::North),
            1 => Some(Actions::South),
            2 => Some(Actions::East),
            3 => Some(Actions::West),
            _ => None,
        }
    }
}

impl Rand for Actions {
    fn rand<R: Rng>(rng: &mut R) -> Actions {

        // let n = rng.gen_range(0, 4);

        // match n {

        //     0 => Actions::North,
        //     1 => Actions::South,
        //     2 => Actions::East,
        //     _ => Actions::West,
        // }

        let n: u8 = rng.gen();

        match n {

            0...63 => Actions::North,
            64...127 => Actions::South,
            128...191 => Actions::East,
            _ => Actions::West,
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

    struct MeasureDistribution {
        mean: f64,
        mean_2: f64,
        count: f64,
    }

    impl MeasureDistribution {
        fn new() -> MeasureDistribution {
            MeasureDistribution {
                mean: 0.0,
                mean_2: 0.0,
                count: 0.0,
            }
        }

        fn add_value(&mut self, v: f64) {
            self.count += 1.0;

            let delta = v - self.mean;
            self.mean += delta / self.count;

            let delta2 = v - self.mean;
            self.mean_2 = delta * delta2;
        }

        fn get_distribution(&self) -> (f64, f64) {

            if self.count < 1.0 {
                (f64::NAN, f64::INFINITY)
            } else if self.count < 2.0 {
                (self.mean, f64::NAN)
            } else {
                let std_dev_sqr = self.mean_2 / (self.count - 1.0);
                (self.mean, std_dev_sqr.sqrt())
            }
        }
    }

    #[test]
    fn distribution() {

        let max_iterations = 1000000;
        let expected_std_dev = 1.0 / (max_iterations as f64).sqrt();

        let mut north_dist = MeasureDistribution::new();
        let mut south_dist = MeasureDistribution::new();
        let mut east_dist = MeasureDistribution::new();
        let mut west_dist = MeasureDistribution::new();

        let mut rng = rand::thread_rng();
        for _ in 0..max_iterations {

            let action: Actions = rng.gen();

            north_dist.add_value(if action == Actions::North { 1.0 } else { 0.0 });
            south_dist.add_value(if action == Actions::South { 1.0 } else { 0.0 });
            east_dist.add_value(if action == Actions::East { 1.0 } else { 0.0 });
            west_dist.add_value(if action == Actions::West { 1.0 } else { 0.0 });
        }

        let max_avg_dev = 4.0 * expected_std_dev;

        println!(
            "expected std dev = {:?} max_dev = {:?}",
            expected_std_dev,
            max_avg_dev
        );

        let (north_avg, north_dev) = north_dist.get_distribution();
        println!("north = {:?}", (north_avg, north_dev));

        let (south_avg, south_dev) = south_dist.get_distribution();
        println!("south = {:?}", (south_avg, south_dev));

        let (east_avg, east_dev) = east_dist.get_distribution();
        println!("east = {:?}", (east_avg, east_dev));

        let (west_avg, west_dev) = west_dist.get_distribution();
        println!("west = {:?}", (west_avg, west_dev));

        assert!(north_avg < 0.25 * (1.0 + max_avg_dev));
        assert!(north_avg > 0.25 * (1.0 - max_avg_dev));
        assert!(north_dev < expected_std_dev * 1.1);

        assert!(south_avg < 0.25 * (1.0 + max_avg_dev));
        assert!(south_avg > 0.25 * (1.0 - max_avg_dev));
        assert!(south_dev < expected_std_dev * 1.1);

        assert!(east_avg < 0.25 * (1.0 + max_avg_dev));
        assert!(east_avg > 0.25 * (1.0 - max_avg_dev));
        assert!(east_dev < expected_std_dev * 1.1);

        assert!(west_avg < 0.25 * (1.0 + max_avg_dev));
        assert!(west_avg > 0.25 * (1.0 - max_avg_dev));
        assert!(west_dev < expected_std_dev * 1.1);

    }
}
