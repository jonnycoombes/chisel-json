extern crate test;

use std::any::Any;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;
use test::Bencher;

use crate::scanner::{Lexeme, Scanner, ScannerMode};

#[test]
fn should_handle_empty_input() {
    let buffer: &[u8] = "".as_bytes();
    let mut reader = BufReader::new(buffer);
    let mut scanner = Scanner::new(&mut reader);
    let eoi = scanner
        .with_mode(ScannerMode::IgnoreWhitespace)
        .consume()
        .unwrap();
    assert_eq!(eoi.lexeme, Lexeme::EndOfInput);
}

#[test]
fn should_handle_general_chars() {
    let buffer: &[u8] = "{   } [  ]+  - : ,   ".as_bytes();
    let mut reader = BufReader::new(buffer);
    let mut scanner = Scanner::new(&mut reader);
    let mut lexemes: Vec<Lexeme> = vec![];

    while let Ok(lex) = scanner.with_mode(ScannerMode::IgnoreWhitespace).consume() {
        lexemes.push(lex.lexeme);
        if lex.lexeme == Lexeme::EndOfInput {
            break;
        }
    }

    assert_eq!(
        lexemes,
        vec![
            Lexeme::LeftBrace,
            Lexeme::RightBrace,
            Lexeme::LeftBracket,
            Lexeme::RightBracket,
            Lexeme::Plus,
            Lexeme::Minus,
            Lexeme::Colon,
            Lexeme::Comma,
            Lexeme::EndOfInput,
        ]
    );
}

#[test]
fn should_report_correct_lookahead_coords() {
    let buffer: &[u8] = "123456789".as_bytes();
    let mut reader = BufReader::new(buffer);
    let mut scanner = Scanner::new(&mut reader);
    for index in 1..=4 {
        _ = scanner.lookahead(index)
    }
    assert_eq!(scanner.back_coords().column, 4);
    let lex = scanner.consume().unwrap();
    assert_eq!(lex.coords.column, 1);
}

#[test]
fn should_handle_whitespace_chars() {
    let buffer: &[u8] = " {  }   \n[]+-:,   ".as_bytes();
    let mut reader = BufReader::new(buffer);
    let mut scanner = Scanner::new(&mut reader);
    let mut lexemes: Vec<Lexeme> = vec![];

    while let Ok(lex) = scanner.with_mode(ScannerMode::ProduceWhitespace).consume() {
        lexemes.push(lex.lexeme);
        if lex.lexeme == Lexeme::EndOfInput {
            break;
        }
    }

    assert_eq!(
        lexemes,
        vec![
            Lexeme::Whitespace(' '),
            Lexeme::LeftBrace,
            Lexeme::Whitespace(' '),
            Lexeme::Whitespace(' '),
            Lexeme::RightBrace,
            Lexeme::Whitespace(' '),
            Lexeme::Whitespace(' '),
            Lexeme::Whitespace(' '),
            Lexeme::NewLine,
            Lexeme::LeftBracket,
            Lexeme::RightBracket,
            Lexeme::Plus,
            Lexeme::Minus,
            Lexeme::Colon,
            Lexeme::Comma,
            Lexeme::Whitespace(' '),
            Lexeme::Whitespace(' '),
            Lexeme::Whitespace(' '),
            Lexeme::EndOfInput,
        ]
    );
}

#[test]
fn should_handle_special_chars() {
    let buffer: &[u8] = "\\\"\' \t".as_bytes();
    let mut reader = BufReader::new(buffer);
    let mut scanner = Scanner::new(&mut reader);
    let mut lexemes: Vec<Lexeme> = vec![];
    while let Ok(lex) = scanner.with_mode(ScannerMode::ProduceWhitespace).consume() {
        lexemes.push(lex.lexeme);
        if lex.lexeme == Lexeme::EndOfInput {
            break;
        }
    }
    assert_eq!(
        lexemes,
        vec![
            Lexeme::Escape,
            Lexeme::DoubleQuote,
            Lexeme::SingleQuote,
            Lexeme::Whitespace(' '),
            Lexeme::Whitespace('\t'),
            Lexeme::EndOfInput,
        ]
    );
}

#[should_panic]
fn lookahead_bounds_check() {
    let buffer: &[u8] = "{}[],:".as_bytes();
    let mut reader = BufReader::new(buffer);
    let mut scanner = Scanner::new(&mut reader);
    assert!(scanner
        .with_mode(ScannerMode::IgnoreWhitespace)
        .lookahead(34)
        .is_err());
    let _ = scanner
        .with_mode(ScannerMode::IgnoreWhitespace)
        .lookahead(0);
}

#[test]
fn scan_small_file() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/json/simple_structure.json");
    let f = File::open(path);
    let mut reader = BufReader::new(f.unwrap());
    let mut scanner = Scanner::new(&mut reader);
    let start = Instant::now();
    while let Ok(lex) = scanner.with_mode(ScannerMode::ProduceWhitespace).consume() {
        if lex.lexeme == Lexeme::EndOfInput {
            break;
        }
    }
    println!("Scanned all UTF-8 in {:?}", start.elapsed());
}

#[bench]
fn scan_small_file_benchmark(bencher: &mut Bencher) {
    bencher.iter(|| {
        scan_small_file();
    })
}

#[bench]
fn scan_complex_file_benchmark(bencher: &mut Bencher) {
    bencher.iter(|| {
        scan_complex_file();
    })
}

#[test]
fn scan_large_file() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/json/events.json");
    let f = File::open(path);
    let mut reader = BufReader::new(f.unwrap());
    let mut scanner = Scanner::new(&mut reader);
    let start = Instant::now();
    while let Ok(lex) = scanner.with_mode(ScannerMode::ProduceWhitespace).consume() {
        if lex.lexeme == Lexeme::EndOfInput {
            break;
        }
    }
    println!("Scanned all UTF-8 in {:?}", start.elapsed());
}

#[test]
fn scan_complex_file() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/json/twitter.json");
    let f = File::open(path);
    let mut reader = BufReader::new(f.unwrap());
    let mut scanner = Scanner::new(&mut reader);
    let start = Instant::now();
    while let Ok(lex) = scanner.with_mode(ScannerMode::ProduceWhitespace).consume() {
        if lex.lexeme == Lexeme::EndOfInput {
            break;
        }
    }
    println!("Scanned all UTF-8 in {:?}", start.elapsed());
}
