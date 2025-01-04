use criterion::{criterion_group, criterion_main, Criterion};

fn prepare_benchmark(c: &mut Criterion) {
    c.bench_function("prepare", |b| {
        b.iter(|| {
            // Add the code you want to benchmark here
        })
    });
}

criterion_group!(benches, prepare_benchmark);
criterion_main!(benches);
