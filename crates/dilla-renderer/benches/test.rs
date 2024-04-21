use criterion::{criterion_group, criterion_main, Criterion};
use dilla_renderer::render;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn test_benchmark(c: &mut Criterion) {
    let root = env::var("CARGO_MANIFEST_DIR").expect("Failed to get root directory");
    // let filename = format!("{root}/tests/output/payload.json");
    let filename = format!("{root}/benches/test.json");
    let mut payload = String::new();
    File::open(&filename)
        .unwrap_or_else(|_| panic!("File not found: {}", filename))
        .read_to_string(&mut payload)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", filename));

    c.bench_function("payload_test", |b| b.iter(|| render(&payload, "json")));
}

criterion_group!(benches, test_benchmark);
criterion_main!(benches);
