use chisel_decoders::utf8::Utf8Decoder;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::BufReader;

fn parse() {
    let f = File::open("fixtures/json/twitter.json").unwrap();
    let reader = BufReader::new(f);
    let decoder = Utf8Decoder::new(reader);
    let mut _count = 0;
    while decoder.decode_next().is_ok() {
        _count += 1;
    }
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("parse twitter extract", |b| b.iter(parse));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
