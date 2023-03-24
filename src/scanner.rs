//! Scanner implementation with lookahead.  The scanning and lexing phases are split into
//! distinct components for no particular reason and so the scanner is just responsible for
//! sourcing individual lexemes which are consumed by the lexer to produce fully formed tokens.
//!
//! The current implementation of the scanner is *not* internally thread safe.
#![allow(unused_variables)]

use crate::parser_coords::ParserCoords;
use crate::parser_errors::*;
use crate::scanner_error;
use chisel_decoders::common::DecoderErrorCode;
use chisel_decoders::utf8::Utf8Decoder;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::io::Read;

/// A lexeme enumeration
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Lexeme {
    /// End of the input
    EndOfInput,
    /// Start of a object
    LeftBrace,
    /// End of a object
    RightBrace,
    /// Start of a array
    LeftBracket,
    /// End of an array
    RightBracket,
    /// Separates pairs
    Colon,
    /// Delineates things
    Comma,
    /// Period which may occur within numbers
    Period,
    /// Double quote
    DoubleQuote,
    /// Single quote
    SingleQuote,
    /// Whitespace
    Whitespace(char),
    /// Newline treated separately from other ws
    NewLine,
    /// Escape character (backslash)
    Escape,
    /// Alphabetic (Unicode) character
    Alphabetic(char),
    /// A non-alphabetic (Unicode) character
    NonAlphabetic(char),
    /// Numeric character
    Digit(char),
    /// The plus character
    Plus,
    /// Minus character
    Minus,
    /// A catch-all for non-recognised characters
    NotRecognised(char),
}

#[macro_export]
macro_rules! is_period {
    ($l:expr) => {
        match $l {
            Lexeme::Period => true,
            _ => false,
        }
    };
}

#[macro_export]
macro_rules! is_digit {
    ($l:expr) => {
        match $l {
            Lexeme::Digit(_) => true,
            _ => false,
        }
    };
}

#[macro_export]
macro_rules! unpack_digit {
    ($l:expr) => {
        match $l {
            Lexeme::Digit(d) => d,
            _ => panic!(),
        }
    };
}

#[macro_export]
macro_rules! is_alphabetic {
    ($l:expr) => {
        match $l {
            Lexeme::Alphabetic(_) => true,
            _ => false,
        }
    };
}

#[macro_export]
macro_rules! unpack_char {
    ($l:expr) => {
        match $l {
            Lexeme::Alphabetic(c) => c,
            _ => panic!(),
        }
    };
}

#[macro_export]
macro_rules! is_whitespace {
    ($l:expr) => {
        match $l {
            Lexeme::Whitespace(_) => true,
            _ => false,
        }
    };
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Structure for packing a lexeme together with it's input coordinates
#[derive(Debug, Copy, Clone)]
pub struct PackedLexeme {
    /// The [Lexeme]
    pub lexeme: Lexeme,
    /// The [InputCoords] for the lexeme
    pub coords: ParserCoords,
}

/// Macro for packing a lexeme and its coordinates into a single structure
macro_rules! packed_lexeme {
    ($l:expr, $c:expr) => {
        PackedLexeme {
            lexeme: $l,
            coords: $c,
        }
    };
}

/// An enumeration to control the handling of whitespace during lexeme lookahead and
/// consumption
#[derive(Debug, Copy, Clone)]
pub enum ScannerMode {
    IgnoreWhitespace,
    ProduceWhitespace,
}

/// A scanner with support for limited lookahead
#[derive(Debug)]
pub struct Scanner<Reader: Read + Debug> {
    /// Lexeme ring buffer, used to implement lookaheads
    buffer: RefCell<VecDeque<PackedLexeme>>,
    /// The stream used for sourcing characters from the input
    decoder: Utf8Decoder<Reader>,
    /// Coordinates of the last lexeme in the lookahead buffer
    back_coords: Cell<ParserCoords>,
    /// Coordinates of the first lexeme in the lookahead buffer
    front_coords: Cell<ParserCoords>,
    /// How whitespace is currently being handled
    mode: Cell<ScannerMode>,
}

impl<Reader: Read + Debug> Scanner<Reader> {
    /// Create a new scanner instance with a given lookahead
    pub fn new(reader: Reader) -> Self {
        Scanner {
            buffer: RefCell::new(VecDeque::new()),
            decoder: Utf8Decoder::new(reader),
            back_coords: Cell::new(ParserCoords::default()),
            front_coords: Cell::new(ParserCoords::default()),
            mode: Cell::new(ScannerMode::IgnoreWhitespace),
        }
    }

    /// Switch the whitespace handling mode within the scanner
    pub fn with_mode(&self, mode: ScannerMode) -> &Self {
        self.mode.replace(mode);
        self
    }

    /// Get the coordinates for the *last* lexeme in the lookahead buffer
    pub fn back_coords(&self) -> ParserCoords {
        self.back_coords.get()
    }

    /// Get the coordinates for the *first* lexeme currently in the lookahead buffer
    pub fn front_coords(&self) -> ParserCoords {
        self.front_coords.get()
    }

    /// Consume the next lexeme from the scanner. Will return a [Utf8DecoderErrorType] if there
    /// are no more lexemes available.  Will produce an EOI (end-of-input) lexeme when
    /// the end of input is reached.
    pub fn consume(&self) -> ParserResult<PackedLexeme> {
        let mut buffer = self.buffer.borrow_mut();
        match buffer.is_empty() {
            false => {
                let lex = buffer.pop_front().unwrap();
                self.front_coords.replace(lex.coords);
                Ok(lex)
            }
            true => match self.char_to_lexeme() {
                Ok(lex) => Ok(lex),
                Err(err) => match err.code {
                    ParserErrorCode::EndOfInput => {
                        Ok(packed_lexeme!(Lexeme::EndOfInput, self.back_coords.get()))
                    }
                    _ => scanner_error!(
                        ParserErrorCode::ExpectedLexeme,
                        "failed to convert a char to a valid lexeme",
                        self.back_coords.get(),
                        err
                    ),
                },
            },
        }
    }

    /// Discard the next `count` lexemes from the input. Return the updated [InputCoords]
    /// for the input
    pub fn discard(&self, count: usize) -> ParserCoords {
        for _ in 1..=count {
            _ = self.consume();
        }
        self.front_coords.get()
    }

    /// Looks ahead in the lexeme stream by a given count. If there are insufficient lexemes
    /// available, then [None] will be returned. This method does not consume any lexemes, it
    /// provides a copy of the lexeme at a specific point in the internal buffer (deque).
    pub fn lookahead(&self, count: usize) -> ParserResult<PackedLexeme> {
        assert!(count > 0);
        let mut error: Option<ParserError> = None;
        while self.buffer.borrow().len() < count {
            match self.char_to_lexeme() {
                Ok(l) => self.buffer.borrow_mut().push_back(l),
                Err(err) => {
                    error = Some(err);
                    break;
                }
            }
        }
        match error {
            None => {
                self.front_coords
                    .replace(self.buffer.borrow().get(0).unwrap().coords);
                Ok(*self.buffer.borrow().get(count - 1).unwrap())
            }
            Some(err) => Err(err),
        }
    }

    /// Advance over any whitespace in the input stream, and try to produce a valid character
    fn advance(&self) -> ParserResult<char> {
        loop {
            match self.decoder.decode_next() {
                Ok(c) => {
                    self.back_coords.replace(ParserCoords {
                        absolute: self.back_coords.get().absolute + 1,
                        line: self.back_coords.get().line,
                        column: self.back_coords.get().column + 1,
                    });

                    if c == '\n' {
                        self.back_coords.replace(ParserCoords {
                            absolute: self.back_coords.get().absolute,
                            line: self.back_coords.get().line + 1,
                            column: 0,
                        });
                    }

                    match self.mode.get() {
                        ScannerMode::IgnoreWhitespace => {
                            if !c.is_whitespace() {
                                break Ok(c);
                            }
                        }
                        ScannerMode::ProduceWhitespace => {
                            break Ok(c);
                        }
                    }
                }
                Err(err) => match err.code {
                    DecoderErrorCode::EndOfInput => {
                        break scanner_error!(ParserErrorCode::EndOfInput, "end of input reached");
                    }
                    _ => {
                        break scanner_error!(
                            ParserErrorCode::StreamFailure,
                            "next_char failed",
                            self.back_coords.get()
                        );
                    }
                },
            }
        }
    }

    /// Take the next character from the underlying stream and attempt conversion into a
    /// valid lexeme. Pack the current [InputCoords] into the return tuple value.
    fn char_to_lexeme(&self) -> ParserResult<PackedLexeme> {
        match self.advance() {
            Ok(c) => match c {
                '{' => Ok(packed_lexeme!(Lexeme::LeftBrace, self.back_coords.get())),
                '}' => Ok(packed_lexeme!(Lexeme::RightBrace, self.back_coords.get())),
                '[' => Ok(packed_lexeme!(Lexeme::LeftBracket, self.back_coords.get())),
                ']' => Ok(packed_lexeme!(Lexeme::RightBracket, self.back_coords.get())),
                '.' => Ok(packed_lexeme!(Lexeme::Period, self.back_coords.get())),
                ':' => Ok(packed_lexeme!(Lexeme::Colon, self.back_coords.get())),
                ',' => Ok(packed_lexeme!(Lexeme::Comma, self.back_coords.get())),
                '\\' => Ok(packed_lexeme!(Lexeme::Escape, self.back_coords.get())),
                '\"' => Ok(packed_lexeme!(Lexeme::DoubleQuote, self.back_coords.get())),
                '\'' => Ok(packed_lexeme!(Lexeme::SingleQuote, self.back_coords.get())),
                '+' => Ok(packed_lexeme!(Lexeme::Plus, self.back_coords.get())),
                '-' => Ok(packed_lexeme!(Lexeme::Minus, self.back_coords.get())),
                '\n' => Ok(packed_lexeme!(Lexeme::NewLine, self.back_coords.get())),
                c if c.is_whitespace() => Ok(packed_lexeme!(
                    Lexeme::Whitespace(c),
                    self.back_coords.get()
                )),
                c if c.is_ascii_digit() => {
                    Ok(packed_lexeme!(Lexeme::Digit(c), self.back_coords.get()))
                }
                c if c.is_alphabetic() => Ok(packed_lexeme!(
                    Lexeme::Alphabetic(c),
                    self.back_coords.get()
                )),
                _ => Ok(packed_lexeme!(
                    Lexeme::NonAlphabetic(c),
                    self.back_coords.get()
                )),
            },
            Err(err) => Err(err),
        }
    }
}
