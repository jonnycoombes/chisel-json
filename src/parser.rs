use chisel_stringtable::btree_string_table::BTreeStringTable;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, Read};
use std::rc::Rc;

use crate::coords::Span;
use crate::errors::{ParserError, ParserErrorCode, ParserResult, ParserStage};
use crate::lexer_old::{Lexer, PackedToken, Token};
use crate::parser_error;
use crate::paths::{PathElement, PathElementStack};
use crate::JsonValue;
use chisel_stringtable::common::StringTable;

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
pub struct Parser<B: BufRead> {
    /// A stack for tracking the current path within the parsed JSON
    path: PathElementStack,
    /// The [Lexer] used by the parser
    lexer: Lexer<B>,
    
    last_span: Span,
}

impl<B: BufRead> Parser<B> {
    pub fn new(input: B) -> Self {
        Parser {
            path: Default::default(),
            lexer: Lexer::new(input),
            last_span: Span::default(),
        }
    }

    pub fn parse(&mut self) -> ParserResult<JsonValue> {
        self.path.push(PathElement::Root);
        self.parse_value()
    }

    fn parse_value(&self) -> ParserResult<JsonValue> {
        match self.lexer.consume()? {
            (Token::StartObject, _) => self.parse_object(),
            (Token::StartArray, _) => self.parse_array(),
            (Token::Str(hash), _) => Ok(JsonValue::String(self.lexer.lookup_string(hash))),
            (Token::Num(value), _) => Ok(JsonValue::Number(value)),
            (Token::Bool(value), _) => Ok(JsonValue::Boolean(value)),
            (Token::Null, _) => Ok(JsonValue::Null),
            (_, span) => {
                parser_error!(
                    ParserErrorCode::UnexpectedToken,
                    format!(
                        "unexpected found whilst attempting to parse valid value: {}",
                        span
                    )
                )
            }
        }
    }

    /// An object is just a list of comma separated KV pairs
    fn parse_object(&self) -> ParserResult<JsonValue> {
        let mut pairs: HashMap<String, JsonValue> = HashMap::new();
        loop {
            match self.lexer.consume()? {
                (Token::Str(hash), _) => {
                    let colon = self.lexer.consume()?;
                    if is_token_colon!(colon) {
                        pairs.insert(
                            self.lexer.lookup_string(hash).to_string(),
                            self.parse_value()?,
                        );
                    } else {
                        return parser_error!(
                            ParserErrorCode::PairExpected,
                            format!(
                                "expected a colon within the input, but didn't find one: {}",
                                colon.1
                            )
                        );
                    }
                }
                (Token::Comma, _) => (),
                (Token::EndObject, _) => return Ok(JsonValue::Object(pairs)),
                (token, span) => {
                    return parser_error!(
                        ParserErrorCode::InvalidObject,
                        format!("invalid object definition found: {}, {:?}", span, token)
                    );
                }
            }
        }
    }

    /// An array is just a list of comma separated values
    fn parse_array(&self) -> ParserResult<JsonValue> {
        let mut values: Vec<JsonValue> = vec![];
        loop {
            match self.lexer.consume()? {
                (Token::EndArray, _) => return Ok(JsonValue::Array(values)),
                (Token::StartObject, _) => values.push(self.parse_object()?),
                (Token::Str(hash), _) => {
                    values.push(JsonValue::String(self.lexer.lookup_string(hash)))
                }
                (Token::Num(value), _) => values.push(JsonValue::Number(value)),
                (Token::Bool(value), _) => values.push(JsonValue::Boolean(value)),
                (Token::Null, _) => values.push(JsonValue::Null),
                (Token::Comma, _) => (),
                (token, span) => {
                    return parser_error!(
                        ParserErrorCode::InvalidArray,
                        format!("invalid object definition found: {}, {:?}", span, token)
                    );
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
        let mut parser = Parser::new(reader);
        let parsed = parser.parse();
        assert!(parsed.is_ok());
    }

    #[test]
    fn should_parse_basic_test_files() {
        for f in fs::read_dir("fixtures/json/valid").unwrap() {
            let path = f.unwrap().path();
            if path.is_file() {
                let len = fs::metadata(&path).unwrap().len();
                let start = Instant::now();
                let reader = reader_from_file!(path.to_str().unwrap());
                let mut parser = Parser::new(reader);
                let parsed = parser.parse();
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
