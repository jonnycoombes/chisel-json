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
            let _ = parser.parse_dom(reader);
        }
    };
}

build_parse_benchmark!(twitter, "twitter");
build_parse_benchmark!(canada, "canada");
build_parse_benchmark!(citm_catalog, "citm_catalog");
build_parse_benchmark!(simple, "simple");
build_parse_benchmark!(schema, "schema");
build_parse_benchmark!(rota, "rota");

fn benchmark_citm_catalog(c: &mut Criterion) {
    c.bench_function("DOM parse of citm_catalog", |b| b.iter(citm_catalog));
}

fn benchmark_twitter(c: &mut Criterion) {
    c.bench_function("DOM parse of twitter", |b| b.iter(twitter));
}

fn benchmark_canada(c: &mut Criterion) {
    c.bench_function("DOM parse of canada", |b| b.iter(canada));
}

fn benchmark_simple(c: &mut Criterion) {
    c.bench_function("DOM parse of simple", |b| b.iter(simple));
}

fn benchmark_schema(c: &mut Criterion) {
    c.bench_function("DOM parse of schema", |b| b.iter(schema));
}

fn benchmark_rota(c: &mut Criterion) {
    c.bench_function("DOM parse of rota", |b| b.iter(rota));
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark_citm_catalog, benchmark_twitter, benchmark_canada, benchmark_simple, benchmark_schema, benchmark_rota
}
criterion_main!(benches);