use criterion::{Criterion, criterion_group, criterion_main};
use reloaded3_localisation::locale_api::parser::parse_r3locale_bytes;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parser", |b| {
        b.iter(|| {
            // Pass the Path reference to the function
            match parse_r3locale_bytes(include_bytes!("../../src/example.r3l")) {
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
