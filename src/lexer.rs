use crate::coords::{Coords, Span};
use crate::errors::{ParserError, ParserErrorCode, ParserResult, ParserStage};
use crate::{lexer_error, parser_error};
use chisel_decoders::common::{DecoderError, DecoderErrorCode, DecoderResult};
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
    buffer: Rc<RefCell<Vec<char>>>,

    /// Current input [Coords]
    coords: Rc<RefCell<Coords>>,
}

impl<B: BufRead> Lexer<B> {
    pub fn new(reader: B) -> Self {
        Lexer {
            decoder: Utf8Decoder::new(reader),
            string_table: Rc::new(RefCell::new(BTreeStringTable::new())),
            buffer: Rc::new(RefCell::new(Vec::with_capacity(DEFAULT_BUFFER_SIZE))),
            coords: Rc::new(RefCell::new(Coords::default())),
        }
    }

    /// Look up a string from the string table, given a [u64] key
    pub fn lookup_string(&self, key: u64) -> Cow<'static, str> {
        self.string_table.borrow().get(key).unwrap().clone()
    }

    /// Consume the next [Token] from the input
    pub fn consume(&self) -> ParserResult<PackedToken> {
        Ok(packed_token!(Token::Null, Coords::default()))
    }

    /// Advance a character in the input stream, and push onto the end of the internal buffer. This
    /// will update the current input [Coords]. Optionally skip whitespace in the input, (but still
    /// update the coordinates accordingly).  
    fn advance(&self, skip_whitespace: bool) -> ParserResult<()> {
        let mut buffer = self.buffer.borrow_mut();
        let mut coords = self.coords.borrow_mut();
        loop {
            match self.decoder.decode_next() {
                Ok(c) => {
                    if c == '\n' {
                        coords.inc(true);
                    } else {
                        coords.inc(false);
                    }
                    if !skip_whitespace {
                        buffer.push(c);
                    } else if !c.is_whitespace() {
                        buffer.push(c)
                    }
                    break;
                }
                Err(err) => {
                    return match err.code {
                        DecoderErrorCode::StreamFailure => {
                            lexer_error!(ParserErrorCode::StreamFailure, err.message)
                        }
                        DecoderErrorCode::InvalidByteSequence => {
                            lexer_error!(ParserErrorCode::NonUtf8InputDetected, err.message)
                        }
                        DecoderErrorCode::EndOfInput => {
                            lexer_error!(ParserErrorCode::EndOfInput, err.message)
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
