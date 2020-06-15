use num_complex::Complex;
use std::f64::consts::PI;

#[allow(non_upper_case_globals)]
const i: Complex<f64> = Complex::new(0.0, 1.0);

fn calculate_kth_nth(x_n: &i16, n: usize, n_samples: usize, k: usize) -> Complex<f64> {
    let x_n = *x_n as f64;
    let n = n as f64;
    let n_samples = n_samples as f64;
    let k = k as f64;
    let inner = 2.0 * PI * k * n / n_samples;
    x_n * (inner.cos() - i * inner.sin())
}

fn calculate_kth(k: usize, samples: &Vec<i16>) -> Complex<f64> {
    let mut x_k = Complex::new(0.0, 0.0);
    let n_samples = samples.len();
    for (n, x_n) in samples.iter().enumerate() {
        let tmp = calculate_kth_nth(x_n, n, n_samples, k);
        x_k += tmp;
    }
    x_k
}

pub fn fourier_transform(samples: Vec<i16>) -> Vec<Complex<f64>> {
    let mut transformed_samples: Vec<Complex<f64>> = Vec::new();
    let n_samples = samples.len();
    for k in 0..n_samples {
        let x_k = calculate_kth(k, &samples);
        transformed_samples.push(x_k);
    }
    transformed_samples
}

fn calculate_kth_nth_inverse(
    x_n: &Complex<f64>,
    n: usize,
    n_samples: usize,
    k: usize,
) -> Complex<f64> {
    //let x_n = *x_n as Complex<f64>;
    let n = n as f64;
    let n_samples = n_samples as f64;
    let k = k as f64;
    let inner = 2.0 * PI * k * n / n_samples;
    x_n * (inner.cos() + i * inner.sin())
}

fn calculate_kth_inverse(k: usize, samples: &Vec<Complex<f64>>) -> Complex<f64> {
    let mut x_k = Complex::new(0.0, 0.0);
    let n_samples = samples.len();
    for (n, x_n) in samples.iter().enumerate() {
        let tmp = calculate_kth_nth_inverse(x_n, n, n_samples, k);
        x_k += tmp;
    }
    x_k
}

pub fn inverse_fourier_transform(samples: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let mut transformed_samples: Vec<Complex<f64>> = Vec::new();
    let n_samples = samples.len();
    for k in 0..n_samples {
        let x_k = calculate_kth_inverse(k, &samples) / n_samples as f64;
        transformed_samples.push(x_k);
    }
    transformed_samples
}

#[cfg(test)]
const INPULSE_AT_ORIGIN: [Complex<f64>; 8] = [
    Complex::new(1.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
];

#[cfg(test)]
const INPULSE_AT_ONE: [Complex<f64>; 8] = [
    Complex::new(0.0, 0.0),
    Complex::new(1.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
];

#[cfg(test)]
fn round_complex(complex: &mut Complex<f64>, sig_figs: usize) {
    let magnitude = 10.0_f64.powf(sig_figs as f64);
    complex.re = (complex.re * magnitude).round() / magnitude;
    complex.im = (complex.im * magnitude).round() / magnitude;
}

#[cfg(test)]
mod there_and_back_again {
    use super::{fourier_transform, inverse_fourier_transform, round_complex, INPULSE_AT_ONE, INPULSE_AT_ORIGIN};

    #[test]
    fn inpulse_at_origin() {
        let input = INPULSE_AT_ORIGIN.to_vec();
        let input_real = input.iter().map(|x| x.re as i16).collect();
        let transformed = fourier_transform(input_real);
        let mut result = inverse_fourier_transform(transformed);
        for x in &mut result {
            round_complex(x, 10)
        }
        assert_eq!(input, result);
    }

    #[test]
    fn inpulse_at_one() {
        let input = INPULSE_AT_ONE.to_vec();
        let input_real = input.iter().map(|x| x.re as i16).collect();
        let transformed = fourier_transform(input_real);
        let mut result = inverse_fourier_transform(transformed);
        for x in &mut result {
            round_complex(x, 10)
        }
        assert_eq!(input, result);
    }
}

#[cfg(test)]
mod ft_test {
    use super::fourier_transform;
    use num_complex::Complex;

    #[test]
    fn impulse_at_origin() {
        let input: Vec<i16> = vec![1, 0, 0, 0, 0, 0, 0, 0];
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
        ];
        let result = fourier_transform(input);
        assert_eq!(expected, result);
    }

    #[test]
    fn impulse_at_one() {
        let input: Vec<i16> = vec![0, 1, 0, 0, 0, 0, 0, 0];
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.707, -0.707),
            Complex::new(0.0, -1.0),
            Complex::new(-0.707, -0.707),
            Complex::new(-1.0, 0.0),
            Complex::new(-0.707, 0.707),
            Complex::new(0.0, 1.0),
            Complex::new(0.707, 0.707),
        ];
        let mut result = fourier_transform(input);
        for x in &mut result {
            round_complex(x, 10)
        }
        assert_eq!(expected, result);
    }
}

#[cfg(test)]
mod ift_test {
    use super::{inverse_fourier_transform, INPULSE_AT_ONE, INPULSE_AT_ORIGIN};
    use num_complex::Complex;

    #[test]
    fn impulse_at_origin() {
        let input = INPULSE_AT_ORIGIN.to_vec();
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
        let result = inverse_fourier_transform(input);
        assert_eq!(expected, result);
    }

    #[test]
    fn impulse_at_one() {
        let input = INPULSE_AT_ONE.to_vec();
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(0.125, 0.0),
            Complex::new(0.088, 0.088),
            Complex::new(0.000, 0.125),
            Complex::new(-0.088, 0.088),
            Complex::new(-0.125, 0.0),
            Complex::new(-0.088, -0.088),
            Complex::new(0.000, -0.125),
            Complex::new(0.088, -0.088),
        ];
        let mut result = inverse_fourier_transform(input);
        for x in &mut result {
            round_complex(x, 10)
        }
        assert_eq!(expected, result);
    }
}
