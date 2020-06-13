use std::f64::consts::PI;

//TODO calculate and return complex part too
fn calculate_kth_nth(x_n: &i16, n: usize, n_samples: usize, k: usize) -> f64 {
    let x_n = *x_n as f64;
    let n = n as f64;
    let n_samples = n_samples as f64;
    let k = k as f64;
    let inner = 2.0 * PI * k * n / n_samples;
    //println!("inner: {}", inner.cos());
    //println!("x_n: {}", x_n);
    x_n * inner.cos()
}

fn calculate_kth(k: usize, samples: &Vec<i16>) -> f64 {
    let mut x_k: f64 = 0.0;
    let n_samples = samples.len();
    for (n, x_n) in samples.iter().enumerate() {
        let tmp = calculate_kth_nth(x_n, n, n_samples, k);
        //println!("tmp: {}", tmp);
        x_k += tmp;
    }
    x_k / n_samples as f64
}

//TODO write a few tests for this
pub fn finite_fourier_transform(samples: Vec<i16>) -> Vec<f64> {
    let mut transformed_samples: Vec<f64> = Vec::new();
    let n_samples = samples.len();
    for k in 0..n_samples {
        let x_k = calculate_kth(k, &samples);
        transformed_samples.push(x_k);
    }
    transformed_samples
}

#[cfg(test)]
mod fft_test {
    use super::finite_fourier_transform;

    #[test]
    fn impulse_at_origin() {
        let zeros: Vec<i16> = vec![1, 0, 0, 0, 0, 0, 0, 0];
        let expected: Vec<f64> = vec![0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125];
        let result = finite_fourier_transform(zeros);
        assert_eq!(expected, result);
    }

    #[test]
    fn impulse_at_one() {
        let zeros: Vec<i16> = vec![0, 1, 0, 0, 0, 0, 0, 0];
        let expected: Vec<f64> = vec![0.125, 0.088, 0.000, -0.088, -0.125, -0.088, 0.000, 0.088];
        let mut result = finite_fourier_transform(zeros);
        result = result
            .iter()
            .map(|x| (x * 1000.0).round() / 1000.0)
            .collect();
        assert_eq!(expected, result);
    }
}
