//! The DOM parser
//!
//! Something
use crate::coords::Coords;
use crate::decoders::{DecoderSelector, Encoding};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::rc::Rc;

use crate::coords::Span;
use crate::dom_parser_error;
use crate::errors::{ParserError, ParserErrorDetails, ParserErrorSource, ParserResult};
use crate::lexer::{Lexer, Token};
use crate::JsonValue;

/// Main JSON parser struct
pub struct Parser {
    decoders: DecoderSelector,
    encoding: Encoding,
}

impl Default for Parser {
    /// The default encoding is Utf-8
    fn default() -> Self {
        Self {
            decoders: Default::default(),
            encoding: Default::default(),
        }
    }
}

impl Parser {
    /// Create a new instance of the parser using a specific [Encoding]
    pub fn with_encoding(encoding: Encoding) -> Self {
        Self {
            decoders: Default::default(),
            encoding,
        }
    }

    ///
    pub fn parse_file<PathLike: AsRef<Path>>(&self, path: PathLike) -> ParserResult<JsonValue> {
        match File::open(&path) {
            Ok(f) => {
                let mut reader = BufReader::new(f);
                let mut chars = self.decoders.new_decoder(&mut reader, self.encoding);
                self.parse(&mut chars)
            }
            Err(_) => {
                dom_parser_error!(ParserErrorDetails::InvalidFile)
            }
        }
    }

    pub fn parse_bytes(&self, bytes: &[u8]) -> ParserResult<JsonValue> {
        let mut reader = BufReader::new(bytes);
        let mut chars = self.decoders.default_decoder(&mut reader);
        self.parse(&mut chars)
    }

    pub fn parse_str(&self, str: &str) -> ParserResult<JsonValue> {
        let mut reader = BufReader::new(str.as_bytes());
        let mut chars = self.decoders.default_decoder(&mut reader);
        self.parse(&mut chars)
    }

    pub fn parse(&self, chars: &mut impl Iterator<Item = char>) -> ParserResult<JsonValue> {
        let mut lexer = Lexer::new(chars);
        match lexer.consume()? {
            (Token::StartObject, _) => self.parse_object(&mut lexer),
            (Token::StartArray, _) => self.parse_array(&mut lexer),
            (_, span) => {
                dom_parser_error!(ParserErrorDetails::InvalidRootObject, span.start)
            }
        }
    }

    fn parse_value(&self, lexer: &mut Lexer) -> ParserResult<JsonValue> {
        match lexer.consume()? {
            (Token::StartObject, _) => self.parse_object(lexer),
            (Token::StartArray, _) => self.parse_array(lexer),
            (Token::Str(str), _) => Ok(JsonValue::String(Cow::Owned(str))),
            (Token::Float(value), _) => Ok(JsonValue::Float(value)),
            (Token::Integer(value), _) => Ok(JsonValue::Integer(value)),
            (Token::Boolean(value), _) => Ok(JsonValue::Boolean(value)),
            (Token::Null, _) => Ok(JsonValue::Null),
            (token, span) => {
                dom_parser_error!(ParserErrorDetails::UnexpectedToken(token), span.start)
            }
        }
    }

    /// An object is just a list of comma separated KV pairs
    fn parse_object(&self, lexer: &mut Lexer) -> ParserResult<JsonValue> {
        let mut pairs = vec![];
        loop {
            match lexer.consume()? {
                (Token::Str(str), _) => {
                    let should_be_colon = lexer.consume()?;
                    match should_be_colon {
                        (Token::Colon, _) => pairs.push((str, self.parse_value(lexer)?)),
                        (_, _) => {
                            return dom_parser_error!(
                                ParserErrorDetails::PairExpected,
                                should_be_colon.1.start
                            )
                        }
                    }
                }
                (Token::Comma, _) => (),
                (Token::EndObject, _) => return Ok(JsonValue::Object(pairs)),
                (_token, span) => {
                    return dom_parser_error!(ParserErrorDetails::InvalidObject, span.start);
                }
            }
        }
    }

    /// An array is just a list of comma separated values
    fn parse_array(&self, lexer: &mut Lexer) -> ParserResult<JsonValue> {
        let mut values: Vec<JsonValue> = vec![];
        loop {
            match lexer.consume()? {
                (Token::StartArray, _) => values.push(self.parse_array(lexer)?),
                (Token::EndArray, _) => return Ok(JsonValue::Array(values)),
                (Token::StartObject, _) => values.push(self.parse_object(lexer)?),
                (Token::Str(str), _) => values.push(JsonValue::String(Cow::Owned(str))),
                (Token::Float(value), _) => values.push(JsonValue::Float(value)),
                (Token::Integer(value), _) => values.push(JsonValue::Integer(value)),
                (Token::Boolean(value), _) => values.push(JsonValue::Boolean(value)),
                (Token::Null, _) => values.push(JsonValue::Null),
                (Token::Comma, _) => (),
                (_token, span) => {
                    return dom_parser_error!(ParserErrorDetails::InvalidArray, span.start);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_macros)]

    use crate::decoders::DecoderSelector;
    use crate::dom::Parser;
    use crate::errors::ParserErrorDetails;
    use crate::relative_file;
    use bytesize::ByteSize;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;
    use std::time::Instant;
    use std::{env, fs};

    #[test]
    fn should_parse_char_iterators_directly() {
        let source = r#"{
            "test" : 1232.0,
            "some other" : "thasdasd",
            "a bool" : true,
            "an array" : [1,2,3,4,5.8,6,7.2,7,8,10]
        }"#;
        let parser = Parser::default();
        let parsed = parser.parse(&mut source.chars());
        println!("{parsed:?}");
        assert!(parsed.is_ok())
    }

    #[test]
    fn should_parse_lengthy_arrays() {
        let path = relative_file!("fixtures/json/valid/bc_block.json");
        let parser = Parser::default();
        let parsed = parser.parse_file(&path);
        println!("{parsed:?}");
        assert!(parsed.is_ok());
    }

    #[test]
    fn should_parse_simple_schema() {
        let path = relative_file!("fixtures/json/valid/simple_schema.json");
        let parser = Parser::default();
        let parsed = parser.parse_file(&path);
        println!("{parsed:?}");
        assert!(parsed.is_ok());
    }
    #[test]
    fn should_successfully_bail() {
        let path = relative_file!("fixtures/json/invalid/invalid_1.json");
        let parser = Parser::default();
        let parsed = parser.parse_file(&path);
        println!("Parse result = {:?}", parsed);
        assert!(parsed.is_err());
        assert!(parsed.err().unwrap().details == ParserErrorDetails::InvalidRootObject);
    }
    #[test]
    fn should_parse_basic_test_files() {
        for f in fs::read_dir("fixtures/json/valid").unwrap() {
            let path = f.unwrap().path();
            println!("Parsing {:?}", &path);
            if path.is_file() {
                let len = fs::metadata(&path).unwrap().len();
                let start = Instant::now();
                let path = relative_file!(path.to_str().unwrap());
                let parser = Parser::default();
                let parsed = parser.parse_file(&path);
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
