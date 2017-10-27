use std::f64;

#[derive(Debug, Default, Clone, Copy)]
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
        self.mean_2 += delta * delta2;
    }

    pub fn add_distribution(&mut self, other: &MeasureDistribution) {

        if other.count > 0.0 {

            if self.count == 0.0 {
                *self = *other;

            } else {

                let total_count = self.count + other.count;
                let new_mean = (self.count * self.mean + other.count * other.mean) / total_count;

                let delta = other.mean - self.mean;

                let new_mean_2 = self.mean_2 + other.mean_2 +
                    (delta * delta * (self.count * other.count / total_count));

                self.mean = new_mean;
                self.mean_2 = new_mean_2;
                self.count = total_count;
            }
        }
    }

    pub fn get_distribution(&self) -> (f64, f64) {

        if self.count < 1.0 {
            (f64::NAN, f64::INFINITY)
        } else if self.count < 2.0 {
            (self.mean, f64::NAN)
        } else {
            let variance = self.mean_2 / (self.count - 1.0);
            (self.mean, variance.sqrt())
        }
    }

    pub fn get_count(&self) -> f64 {
        self.count
    }
}
