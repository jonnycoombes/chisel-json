use chisel_json::parser::Parser;
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use std::fs::File;
use std::io::BufReader;

macro_rules! build_parse_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/json/bench/{}.json", $filename)).unwrap();
            let reader = BufReader::new(f);
            let parser = Parser::default();
            let _ = parser.parse(reader);
        }
    };
}

build_parse_benchmark!(twitter, "twitter");
build_parse_benchmark!(canada, "canada");
build_parse_benchmark!(citm_catalog, "citm_catalog");

fn benchmark_citm_catalog(c: &mut Criterion) {
    c.bench_function("parse of citm_catalog", |b| b.iter(citm_catalog));
}

fn benchmark_twitter(c: &mut Criterion) {
    c.bench_function("parse of twitter", |b| b.iter(twitter));
}

fn benchmark_canada(c: &mut Criterion) {
    c.bench_function("parse of canada", |b| b.iter(canada));
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark_citm_catalog, benchmark_twitter, benchmark_canada
}
criterion_main!(benches);
