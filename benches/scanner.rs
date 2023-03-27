use chisel_json::scanner::{Lexeme, Scanner, ScannerMode};
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::BufReader;

fn scan() {
    let f = File::open("fixtures/json/twitter.json").unwrap();
    let reader = BufReader::new(f);
    let scanner = Scanner::new(reader);
    loop {
        let packed = scanner.with_mode(ScannerMode::ProduceWhitespace).consume();
        if let Ok(lex) = packed {
            if lex.lexeme == Lexeme::EndOfInput {
                break;
            }
        }
    }
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("scan twitter extract", |b| b.iter(scan));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
