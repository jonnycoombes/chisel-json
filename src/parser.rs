use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, Read};
use std::rc::Rc;

use crate::coords::Span;
use crate::errors::{Details, ParserError, ParserResult, Stage};
use crate::lexer::{Lexer, Token};
use crate::parser_error;
use crate::paths::{PathElement, PathElementStack};
use crate::JsonValue;

/// Check whether a packed token contains a colon token
macro_rules! is_token_colon {
    ($t:expr) => {
        match $t {
            (Token::Colon, _) => true,
            (_, _) => false,
        }
    };
}

/// Main JSON parser struct
#[derive(Default)]
pub struct Parser {
    /// A stack for tracking the current path within the parsed JSON
    path: PathElementStack,
    last_span: Span,
}

impl Parser {
    pub fn parse<B: BufRead>(&self, input: B) -> ParserResult<JsonValue> {
        let mut lexer = Lexer::new(input);
        self.parse_value(&mut lexer)
    }

    fn parse_value<B: BufRead>(&self, lexer: &mut Lexer<B>) -> ParserResult<JsonValue> {
        match lexer.consume()? {
            (Token::StartObject, _) => self.parse_object(lexer),
            (Token::StartArray, _) => self.parse_array(lexer),
            (Token::Str(str), _) => Ok(JsonValue::String(Cow::Owned(str))),
            (Token::Num(value), _) => Ok(JsonValue::Number(value)),
            (Token::Bool(value), _) => Ok(JsonValue::Boolean(value)),
            (Token::Null, _) => Ok(JsonValue::Null),
            (_, span) => {
                parser_error!(Details::UnexpectedToken, span.start)
            }
        }
    }

    /// An object is just a list of comma separated KV pairs
    fn parse_object<B: BufRead>(&self, lexer: &mut Lexer<B>) -> ParserResult<JsonValue> {
        let mut pairs = vec![];
        loop {
            match lexer.consume()? {
                (Token::Str(str), _) => {
                    let colon = lexer.consume()?;
                    if is_token_colon!(colon) {
                        pairs.push((str, self.parse_value(lexer)?));
                    } else {
                        return parser_error!(Details::PairExpected, colon.1.start);
                    }
                }
                (Token::Comma, _) => (),
                (Token::EndObject, _) => return Ok(JsonValue::Object(pairs)),
                (_token, span) => {
                    return parser_error!(Details::InvalidObject, span.start);
                }
            }
        }
    }

    /// An array is just a list of comma separated values
    fn parse_array<B: BufRead>(&self, lexer: &mut Lexer<B>) -> ParserResult<JsonValue> {
        let mut values: Vec<JsonValue> = vec![];
        loop {
            match lexer.consume()? {
                (Token::StartArray, _) => values.push(self.parse_array(lexer)?),
                (Token::EndArray, _) => return Ok(JsonValue::Array(values)),
                (Token::StartObject, _) => values.push(self.parse_object(lexer)?),
                (Token::Str(str), _) => values.push(JsonValue::String(Cow::Owned(str))),
                (Token::Num(value), _) => values.push(JsonValue::Number(value)),
                (Token::Bool(value), _) => values.push(JsonValue::Boolean(value)),
                (Token::Null, _) => values.push(JsonValue::Null),
                (Token::Comma, _) => (),
                (_token, span) => {
                    return parser_error!(Details::InvalidArray, span.start);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_macros)]

    use crate::parser::Parser;
    use crate::{reader_from_file, reader_from_relative_file};
    use bytesize::ByteSize;
    use std::fs::File;
    use std::io::BufReader;
    use std::time::Instant;
    use std::{env, fs};

    #[test]
    fn should_parse_lengthy_arrays() {
        let reader = reader_from_file!("fixtures/json/valid/bc_block.json");
        let parser = Parser::default();
        let parsed = parser.parse(reader);
        assert!(parsed.is_ok());
    }

    #[test]
    fn should_parse_basic_test_files() {
        for f in fs::read_dir("fixtures/json/valid").unwrap() {
            let path = f.unwrap().path();
            println!("Parsing {:?}", &path);
            if path.is_file() {
                let len = fs::metadata(&path).unwrap().len();
                let start = Instant::now();
                let reader = reader_from_file!(path.to_str().unwrap());
                let parser = Parser::default();
                let parsed = parser.parse(reader);
                if parsed.is_err() {
                    println!("Parse of {:?} failed!", &path);
                    println!("Parse failed with errors: {:?}", &parsed)
                }
                assert!(parsed.is_ok());
                println!(
                    "Parsed {} in {:?} [{:?}]",
                    ByteSize(len),
                    start.elapsed(),
                    path,
                );
            }
        }
    }
}
