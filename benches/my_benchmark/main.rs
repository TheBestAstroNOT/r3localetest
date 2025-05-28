use criterion::{criterion_group, criterion_main, Criterion};

// Import your crate library code (assuming your crate is named `your_crate_name`)
use r3localetest::{parse_keys_simd_bracketonly};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_keys_simd_bracketonly", |b| {
        b.iter(|| parse_keys_simd_bracketonly())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
