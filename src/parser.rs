use chisel_stringtable::btree_string_table::BTreeStringTable;
use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Read;
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
    pub fn parse<Reader: Debug + Read>(&self, input: Reader) {
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
                    println!("Error encountered during parse: {:?}", err)
                }
            }
        }
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
    use crate::reader_from_relative_file;
    use std::env;
    use std::fs::File;
    use std::io::BufReader;
    use std::time::Instant;

    #[test]
    fn should_provide_a_sensible_default() {
        let parser = Parser::default();
        assert_eq!(0, parser.strings.borrow().len());
    }

    #[test]
    fn should_parse_simple_json() {
        let start = Instant::now();
        let reader = reader_from_relative_file!("fixtures/samples/json/simple_structure.json");
        let parser = Parser::default();
        parser.parse(reader);
        println!("Parsed in {:?}", start.elapsed());
    }
}
