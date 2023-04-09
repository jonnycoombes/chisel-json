use chisel_stringtable::btree_string_table::BTreeStringTable;
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::io::{BufRead, Read};
use std::rc::Rc;

use crate::errors::{ParserError, ParserErrorCode, ParserResult, ParserStage};
use crate::lexer::{Lexer, PackedToken, Token};
use crate::paths::{PathElement, PathElementStack};
use crate::{packed_token_to_pair, parser_error};
use chisel_stringtable::common::StringTable;

/// Main JSON parser struct
pub struct Parser {
    /// [StringTable] used to intern strings during parsing
    strings: Rc<RefCell<dyn StringTable<'static, u64>>>,
    /// A stack for tracking the current path within the parsed JSON
    path: PathElementStack,
}

impl Parser {
    pub fn parse<B: BufRead>(&mut self, input: B) -> ParserResult<()> {
        let mut lexer = Lexer::new(self.strings.clone(), input);
        self.path.push(PathElement::Root);
        match packed_token_to_pair!(lexer.consume()?) {
            (Token::StartObject, _) => self.parse_object(),
            (Token::StartArray, _) => self.parse_array(),
            (_, span) => parser_error!(
                ParserErrorCode::UnexpectedToken,
                format!(
                    "unexpected token found whilst looking for object or array: {}",
                    span
                )
            ),
        }
    }

    fn parse_object(&self) -> ParserResult<()> {
        Ok(())
    }

    fn parse_array(&self) -> ParserResult<()> {
        Ok(())
    }

    fn parse_pair(&self) -> ParserResult<()> {
        Ok(())
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            strings: Rc::new(RefCell::new(BTreeStringTable::new())),
            path: PathElementStack::default(),
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
    fn should_parse_basic_test_files() {
        for f in fs::read_dir("fixtures/json/valid").unwrap() {
            let path = f.unwrap().path();
            if path.is_file() {
                let len = fs::metadata(&path).unwrap().len();
                let start = Instant::now();
                let reader = reader_from_file!(path.to_str().unwrap());
                let mut parser = Parser::default();
                assert!(parser.parse(reader).is_ok());
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
