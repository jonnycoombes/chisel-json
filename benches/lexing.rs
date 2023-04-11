use chisel_json::lexer::{Lexer, PackedToken, Token};
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::BufReader;

macro_rules! build_lex_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/json/valid/{}.json", $filename)).unwrap();
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
                        ()
                    }
                }
            }
        }
    };
}

build_lex_benchmark!(blog_entries, "blog_entries");
build_lex_benchmark!(simple_structure, "simple_structure");
build_lex_benchmark!(bc_block, "bc_block");
build_lex_benchmark!(gh_emojis, "gh_emojis");
build_lex_benchmark!(historical_events, "historical_events");

fn benchmark_blog_entries(c: &mut Criterion) {
    c.bench_function("lex of blog_entries", |b| b.iter(blog_entries));
}

fn benchmark_simple_structure(c: &mut Criterion) {
    c.bench_function("lex of simple_structure", |b| b.iter(simple_structure));
}

fn benchmark_bc_block(c: &mut Criterion) {
    c.bench_function("lex of bc_block", |b| b.iter(bc_block));
}

fn benchmark_gh_emojis(c: &mut Criterion) {
    c.bench_function("lex of gh_emojis", |b| b.iter(gh_emojis));
}

fn benchmark_historical_events(c: &mut Criterion) {
    c.bench_function("lex of historical events", |b| b.iter(historical_events));
}
criterion_group!(
    benches,
    benchmark_blog_entries,
    benchmark_simple_structure,
    benchmark_bc_block,
    benchmark_gh_emojis,
    benchmark_historical_events
);
criterion_main!(benches);
