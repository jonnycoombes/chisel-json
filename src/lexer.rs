use crate::coords::{Coords, Span};
use crate::errors::ParserResult;
use chisel_decoders::utf8::Utf8Decoder;
use chisel_stringtable::btree_string_table::BTreeStringTable;
use chisel_stringtable::common::StringTable;
use std::borrow::Cow;
use std::cell::RefCell;
use std::io::BufRead;
use std::rc::Rc;

/// Default lookahead buffer size
const DEFAULT_BUFFER_SIZE: usize = 0xff;

/// Enumeration of valid JSON tokens
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    StartObject,
    EndObject,
    StartArray,
    EndArray,
    Colon,
    Comma,
    Str(u64),
    Num(f64),
    Null,
    Bool(bool),
    EndOfInput,
}

/// A packed token consists of a [Token] and the [Span] associated with it
pub type PackedToken = (Token, Span);

/// Convenience macro for packing tokens along with their positional information
macro_rules! packed_token {
    ($t:expr, $s:expr, $e:expr) => {
        ($t, Span { start: $s, end: $e })
    };
    ($t:expr, $s:expr) => {
        ($t, Span { start: $s, end: $s })
    };
}

pub struct Lexer<B: BufRead> {
    /// The input [Utf8Decoder]
    decoder: Utf8Decoder<B>,

    /// The [StringTable]
    string_table: Rc<RefCell<dyn StringTable<'static, u64>>>,

    /// Lookahead buffer
    buffer: Vec<char>,
}

impl<B: BufRead> Lexer<B> {
    pub fn new(reader: B) -> Self {
        Lexer {
            decoder: Utf8Decoder::new(reader),
            string_table: Rc::new(RefCell::new(BTreeStringTable::new())),
            buffer: Vec::with_capacity(DEFAULT_BUFFER_SIZE),
        }
    }

    pub fn lookup_string(&self, key: u64) -> Cow<'static, str> {
        self.string_table.borrow().get(key).unwrap().clone()
    }

    pub fn consume(&self) -> ParserResult<PackedToken> {
        Ok(packed_token!(Token::Null, Coords::default()))
    }
}
