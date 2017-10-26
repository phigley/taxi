use std::f64;

#[derive(Default)]
pub struct MeasureDistribution {
    mean: f64,
    mean_2: f64,
    count: f64,
}

impl MeasureDistribution {
    pub fn add_value(&mut self, v: f64) {
        self.count += 1.0;

        let delta = v - self.mean;
        self.mean += delta / self.count;

        let delta2 = v - self.mean;
        self.mean_2 = delta * delta2;
    }

    pub fn get_distribution(&self) -> (f64, f64) {

        if self.count < 1.0 {
            (f64::NAN, f64::INFINITY)
        } else if self.count < 2.0 {
            (self.mean, f64::NAN)
        } else {
            let std_dev_sqr = self.mean_2 / self.count;
            (self.mean, std_dev_sqr.sqrt())
        }
    }
}
