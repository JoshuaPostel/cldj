use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cldj::transform::fourier_transform;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fft", |b| {
        b.iter(|| fourier_transform(black_box(vec![1, 0, 0, 0, 0, 0, 0, 0])))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
