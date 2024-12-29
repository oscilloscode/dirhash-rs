use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::thread;
use std::time::Duration;

pub fn parse_benchmark(c: &mut Criterion) {
    c.bench_function("sleep 10ms", |b| {
        b.iter(|| thread::sleep(Duration::from_millis(10)))
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
