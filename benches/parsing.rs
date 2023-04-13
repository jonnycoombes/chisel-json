use chisel_json::parser::Parser;
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use std::fs::File;
use std::io::BufReader;

macro_rules! build_parse_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/json/valid/{}.json", $filename)).unwrap();
            let reader = BufReader::new(f);
            let parser = Parser::default();
            let _ = parser.parse(reader);
        }
    };
}

build_parse_benchmark!(blog_entries, "blog_entries");
build_parse_benchmark!(simple_structure, "simple_structure");
build_parse_benchmark!(bc_block, "bc_block");
build_parse_benchmark!(gh_emojis, "gh_emojis");
build_parse_benchmark!(historical_events, "historical_events");
build_parse_benchmark!(events, "events");
build_parse_benchmark!(twitter, "twitter");

fn benchmark_blog_entries(c: &mut Criterion) {
    c.bench_function("parse of blog_entries", |b| b.iter(blog_entries));
}

fn benchmark_simple_structure(c: &mut Criterion) {
    c.bench_function("parse of simple_structure", |b| b.iter(simple_structure));
}

fn benchmark_bc_block(c: &mut Criterion) {
    c.bench_function("parse of bc_block", |b| b.iter(bc_block));
}

fn benchmark_gh_emojis(c: &mut Criterion) {
    c.bench_function("parse of gh_emojis", |b| b.iter(gh_emojis));
}

fn benchmark_historical_events(c: &mut Criterion) {
    c.bench_function("parse of historical events", |b| b.iter(historical_events));
}

fn benchmark_events(c: &mut Criterion) {
    c.bench_function("parse of events", |b| b.iter(events));
}

fn benchmark_twitter(c: &mut Criterion) {
    c.bench_function("parse of twitter", |b| b.iter(twitter));
}
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark_blog_entries,
    benchmark_simple_structure,
    benchmark_bc_block,
    benchmark_gh_emojis,
    benchmark_historical_events,
    benchmark_events,
    benchmark_twitter
}
criterion_main!(benches);
