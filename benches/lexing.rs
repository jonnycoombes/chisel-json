use chisel_json::lexer::{Lexer, Token};
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use std::fs::File;
use std::io::BufReader;

macro_rules! build_lex_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/json/bench/{}.json", $filename)).unwrap();
            let reader = BufReader::new(f);
            let mut lexer = Lexer::new(reader);
            loop {
                match lexer.consume() {
                    Ok(t) => {
                        if t.0 == Token::EndOfInput {
                            break;
                        }
                    }
                    Err(err) => {
                        println!("error occurred: {:?}", err);
                    }
                }
            }
        }
    };
}

build_lex_benchmark!(canada, "canada");
build_lex_benchmark!(citm_catalog, "citm_catalog");
build_lex_benchmark!(twitter, "twitter");
build_lex_benchmark!(simple, "simple");

fn benchmark_canada(c: &mut Criterion) {
    c.bench_function("lex of canada", |b| b.iter(canada));
}
fn benchmark_citm_catalog(c: &mut Criterion) {
    c.bench_function("lex of citm_catalog", |b| b.iter(citm_catalog));
}
fn benchmark_twitter(c: &mut Criterion) {
    c.bench_function("lex of twitter", |b| b.iter(twitter));
}
fn benchmark_simple(c: &mut Criterion) {
    c.bench_function("lex of simple", |b| b.iter(simple));
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets= benchmark_twitter, benchmark_citm_catalog, benchmark_canada, benchmark_simple
}
criterion_main!(benches);
