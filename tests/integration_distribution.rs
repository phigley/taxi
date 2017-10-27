
extern crate float_cmp;
extern crate rand;
extern crate taxi;

use float_cmp::ApproxEqUlps;

use taxi::distribution::MeasureDistribution;

#[test]
fn measures_simple() {

    let samples: Vec<usize> = (0..101).collect();

    let mut measurement = MeasureDistribution::default();

    let mut sum = 0.0;
    let mut sum_sqr = 0.0;

    for s in &samples {

        let val = *s as f64;

        measurement.add_value(val);


        sum += val;
        sum_sqr += val * val;
    }

    let total_count = samples.len() as f64;
    let naive_var = (sum_sqr - sum * sum / total_count) / (total_count - 1.0);

    let naive_std_dev = naive_var.sqrt();

    let (mean, std_dev) = measurement.get_distribution();

    println!(
        "mean = {}, std_dev = {} naive_std_dev = {}",
        mean,
        std_dev,
        naive_std_dev,
    );

    assert!(measurement.get_count().approx_eq_ulps(&total_count, 1));
    assert!(mean.approx_eq_ulps(&50.0, 1));
    assert!(std_dev.approx_eq_ulps(&naive_std_dev, 3));
}

#[test]
fn measure_combines_with_empty() {

    let samples: Vec<usize> = (0..101).collect();

    let mut measurement = MeasureDistribution::default();

    for s in samples {
        measurement.add_value(s as f64);
    }

    let (expected_mean, expected_std_dev) = measurement.get_distribution();

    let empty_measurement = MeasureDistribution::default();
    measurement.add_distribution(&empty_measurement);

    let (mean, std_dev) = measurement.get_distribution();

    assert!(mean.approx_eq_ulps(&expected_mean, 1));
    assert!(std_dev.approx_eq_ulps(&expected_std_dev, 3));
}

#[test]
fn measure_empty_combines_with_nonempty() {

    let samples: Vec<usize> = (0..101).collect();

    let mut measurement = MeasureDistribution::default();

    for s in samples {
        measurement.add_value(s as f64);
    }

    let (expected_mean, expected_std_dev) = measurement.get_distribution();

    let mut empty_measurement = MeasureDistribution::default();
    empty_measurement.add_distribution(&measurement);

    let (mean, std_dev) = empty_measurement.get_distribution();

    assert!(mean.approx_eq_ulps(&expected_mean, 1));
    assert!(std_dev.approx_eq_ulps(&expected_std_dev, 3));
}


#[test]
fn measures_combine() {

    let samples: Vec<usize> = (0..101).collect();

    let mut base_line = MeasureDistribution::default();

    for s in &samples {
        base_line.add_value(*s as f64);
    }

    let (base_line_mean, base_line_std_dev) = base_line.get_distribution();

    let combos = [
    	vec![ 50, 50 ],
    	vec![ 10, 10, 10, 10, 10, 10, 10, 10, 10, 10],
    	vec![20, 30, 40, 10],
    	vec![ 80, 5, 9, 3, 2, 1],
    	vec![ 20, 60, 20 ],
    	vec![ 30, 40 ], // Intentionally short of count
    ];

    for combo in &combos {

        let mut sampler = samples.as_slice();

        let mut dists = Vec::with_capacity(combo.len());

        for count in combo {

            let mut dist = MeasureDistribution::default();

            let (current_chunk, remaining) = sampler.split_at(*count);

            for s in current_chunk {
                dist.add_value(*s as f64);
            }

            dists.push(dist);

            sampler = remaining;
        }

        let mut result = MeasureDistribution::default();

        for d in &dists {
            result.add_distribution(d);
        }


        for s in sampler {
            result.add_value(*s as f64);
        }

        let (result_mean, result_std_dev) = result.get_distribution();

        assert!(result_mean.approx_eq_ulps(&base_line_mean, 1));
        assert!(result_std_dev.approx_eq_ulps(&base_line_std_dev, 3));

    }
}
