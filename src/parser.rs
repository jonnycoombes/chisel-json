use chisel_stringtable::btree_string_table::BTreeStringTable;
use std::cell::RefCell;
use std::fmt::Debug;
use std::io::{BufRead, Read};
use std::rc::Rc;

use crate::errors::ParserResult;
use crate::lexer::{Lexer, Token};
use chisel_stringtable::common::StringTable;

/// Main JSON parser struct
pub struct Parser {
    /// [StringTable] used to intern strings during parsing
    strings: Rc<RefCell<dyn StringTable<'static, u64>>>,
}

impl Parser {
    pub fn parse<B: BufRead>(&self, input: B) -> ParserResult<()> {
        let mut lexer = Lexer::new(self.strings.clone(), input);
        loop {
            let token = lexer.consume();
            match token {
                Ok(packed) => match packed.token {
                    Token::StartObject => (),
                    Token::EndObject => (),
                    Token::StartArray => (),
                    Token::EndArray => (),
                    Token::Colon => (),
                    Token::Comma => (),
                    Token::Str(_) => (),
                    Token::Num(_) => (),
                    Token::Null => (),
                    Token::Bool(_) => (),
                    Token::EndOfInput => break,
                },
                Err(err) => {
                    println!("Error encountered during parse: {:?}", err);
                    return Err(err);
                }
            }
        }
        Ok(())
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            strings: Rc::new(RefCell::new(BTreeStringTable::new())),
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
        for f in fs::read_dir("fixtures/json").unwrap() {
            let path = f.unwrap().path();
            if path.is_file() {
                let len = fs::metadata(&path).unwrap().len();
                let start = Instant::now();
                let reader = reader_from_file!(path.to_str().unwrap());
                let parser = Parser::default();
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
