use chisel_json::parser::Parser;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::BufReader;

fn parse() {
    let f = File::open("fixtures/json/blog_entries.json").unwrap();
    let reader = BufReader::new(f);
    let parser = Parser::default();
    let _ = parser.parse(reader);
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("parse blog entries", |b| b.iter(parse));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
