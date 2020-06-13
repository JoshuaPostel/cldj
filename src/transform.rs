use num_complex::Complex;
use std::f64::consts::PI;

fn calculate_kth_nth(x_n: &i16, n: usize, n_samples: usize, k: usize) -> Complex<f64> {
    let x_n = *x_n as f64;
    let n = n as f64;
    let n_samples = n_samples as f64;
    let k = k as f64;
    let inner = 2.0 * PI * k * n / n_samples;
    Complex::new(x_n * inner.cos(), -x_n * inner.sin())
}

fn calculate_kth(k: usize, samples: &Vec<i16>) -> Complex<f64> {
    let mut x_k = Complex::new(0.0, 0.0);
    let n_samples = samples.len();
    for (n, x_n) in samples.iter().enumerate() {
        let tmp = calculate_kth_nth(x_n, n, n_samples, k);
        x_k += tmp;
    }
    x_k / n_samples as f64
}

pub fn finite_fourier_transform(samples: Vec<i16>) -> Vec<Complex<f64>> {
    let mut transformed_samples: Vec<Complex<f64>> = Vec::new();
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
    use num_complex::Complex;

    //source: http://www.sccon.ca/sccon/fft/fft3.htm
    #[test]
    fn impulse_at_origin() {
        let zeros: Vec<i16> = vec![1, 0, 0, 0, 0, 0, 0, 0];
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
        ];
        let result = finite_fourier_transform(zeros);
        assert_eq!(expected, result);
    }

    //source: http://www.sccon.ca/sccon/fft/fft3.htm
    #[test]
    fn impulse_at_one() {
        let zeros: Vec<i16> = vec![0, 1, 0, 0, 0, 0, 0, 0];
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(0.125, 0.0),
            Complex::new(0.088, -0.088),
            Complex::new(0.000, -0.125),
            Complex::new(-0.088, -0.088),
            Complex::new(-0.125, 0.0),
            Complex::new(-0.088, 0.088),
            Complex::new(0.000, 0.125),
            Complex::new(0.088, 0.088),
        ];
        let mut result = finite_fourier_transform(zeros);
        for x in &mut result {
            x.re = (x.re * 1000.0).round() / 1000.0;
            x.im = (x.im * 1000.0).round() / 1000.0;
        }
        assert_eq!(expected, result);
    }
}
