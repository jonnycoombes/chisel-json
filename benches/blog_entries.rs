use chisel_decoders::utf8::Utf8Decoder;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::BufReader;

fn parse() {
    let f = File::open("fixtures/json/blog_entries.json").unwrap();
    let reader = BufReader::new(f);
    let decoder = Utf8Decoder::new(reader);
    let mut _count = 0;
    while decoder.decode_next().is_ok() {
        _count += 1;
    }
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("parse blog entries", |b| b.iter(parse));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
