use criterion::{Criterion, criterion_group, criterion_main};
use r3localetest::locale_api::get_locale_table_rust;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parser", |b| {
        b.iter(|| {
            // Pass the Path reference to the function
            match get_locale_table_rust(None) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to parse locale table: {:?}", e);
                }
            };
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
