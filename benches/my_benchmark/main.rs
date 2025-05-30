use criterion::{criterion_group, criterion_main, Criterion};
use r3localetest::locale_system::parse_r3locale_file;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parser", |b| {
        b.iter(|| {
            // Pass the Path reference to the function
            parse_r3locale_file(None)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
