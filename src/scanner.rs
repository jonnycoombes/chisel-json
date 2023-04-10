//! Scanner implementation with lookahead.  The scanning and lexing phases are split into
//! distinct components for no particular reason and so the scanner is just responsible for
//! sourcing individual lexemes which are consumed by the lexer to produce fully formed tokens.
//!
//! The current implementation of the scanner is *not* internally thread safe.
#![allow(unused_variables)]

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::io::{BufRead, Read};

use chisel_decoders::common::DecoderErrorCode;
use chisel_decoders::utf8::Utf8Decoder;

use crate::coords::Coords;
use crate::errors::*;
use crate::scanner_error;

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

/// Macro to quickly check whether we have an unrecognised character
#[macro_export]
macro_rules! is_not_recognised {
    ($l:expr) => {
        match $l {
            Lexeme::NotRecognised(_) => true,
            _ => false,
        }
    };
}

/// Macro to quickly check whether we have a digit
#[macro_export]
macro_rules! is_digit {
    ($l:expr) => {
        match $l {
            Lexeme::Digit(_) => true,
            _ => false,
        }
    };
}

/// Macro to extract the character from inside a digit. Note not safe,  will panic.
#[macro_export]
macro_rules! unpack_digit {
    ($l:expr) => {
        match $l {
            Lexeme::Digit(d) => d,
            _ => panic!(),
        }
    };
}

/// Macro to quickly check whether we have an alphabetic character
#[macro_export]
macro_rules! is_alphabetic {
    ($l:expr) => {
        match $l {
            Lexeme::Alphabetic(_) => true,
            _ => false,
        }
    };
}

/// Macro to quickly check whether we have an alphabetic character
#[macro_export]
macro_rules! is_non_alphabetic {
    ($l:expr) => {
        match $l {
            Lexeme::NonAlphabetic(_) => true,
            _ => false,
        }
    };
}

/// Macro to quickly check whether we have an alphabetic character
#[macro_export]
macro_rules! is_whitespace {
    ($l:expr) => {
        match $l {
            Lexeme::Whitespace(_) => true,
            _ => false,
        }
    };
}

/// Macro to extract the character from inside a alphabetic. Note not safe,  will panic.
#[macro_export]
macro_rules! unpack_char {
    ($l:expr) => {
        match $l {
            Lexeme::Alphabetic(c) => c,
            _ => panic!(),
        }
    };
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A packed lexeme is just a pair consisting of a [Lexeme] and its [Coords]
pub type PackedLexeme = (Lexeme, Coords);

/// Macro for packing a lexeme and its coordinates into a single structure
macro_rules! packed_lexeme {
    ($l:expr, $c:expr) => {
        ($l, $c)
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
#[derive()]
pub struct Scanner<B: BufRead> {
    /// Lexeme ring buffer, used to implement lookaheads
    buffer: RefCell<VecDeque<PackedLexeme>>,
    /// The stream used for sourcing characters from the input
    decoder: Utf8Decoder<B>,
    /// Coordinates of the last lexeme in the lookahead buffer
    back_coords: Cell<Coords>,
    /// Coordinates of the first lexeme in the lookahead buffer
    front_coords: Cell<Coords>,
    /// How whitespace is currently being handled
    mode: Cell<ScannerMode>,
}

impl<B: BufRead> Scanner<B> {
    /// Create a new scanner instance with a given lookahead
    pub fn new(reader: B) -> Self {
        Scanner {
            buffer: RefCell::new(VecDeque::new()),
            decoder: Utf8Decoder::new(reader),
            back_coords: Cell::new(Coords::default()),
            front_coords: Cell::new(Coords::default()),
            mode: Cell::new(ScannerMode::IgnoreWhitespace),
        }
    }

    /// Switch the whitespace handling mode within the scanner
    pub fn with_mode(&self, mode: ScannerMode) -> &Self {
        self.mode.replace(mode);
        self
    }

    /// Get the coordinates for the *last* lexeme in the lookahead buffer
    pub fn back_coords(&self) -> Coords {
        self.back_coords.get()
    }

    /// Get the coordinates for the *first* lexeme currently in the lookahead buffer
    pub fn front_coords(&self) -> Coords {
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
                self.front_coords.replace(lex.1);
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
    pub fn discard(&self, count: usize) -> Coords {
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
        let mut buffer = self.buffer.borrow_mut();
        while buffer.len() < count {
            match self.char_to_lexeme() {
                Ok(l) => buffer.push_back(l),
                Err(err) => {
                    error = Some(err);
                    break;
                }
            }
        }
        match error {
            None => {
                self.front_coords.replace(buffer.get(0).unwrap().1);
                Ok(*buffer.get(count - 1).unwrap())
            }
            Some(err) => Err(err),
        }
    }

    /// Advance over any whitespace in the input stream, and try to produce a valid character
    fn advance(&self) -> ParserResult<char> {
        loop {
            match self.decoder.decode_next() {
                Ok(c) => {
                    self.back_coords.replace(Coords {
                        absolute: self.back_coords.get().absolute + 1,
                        line: self.back_coords.get().line,
                        column: self.back_coords.get().column + 1,
                    });

                    if c == '\n' {
                        self.back_coords.replace(Coords {
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

#[cfg(test)]
mod tests {
    #![allow(unused_macros)]

    use std::fs::File;
    use std::io::BufReader;
    use std::time::Instant;
    use std::{env, fs};

    use bytesize::ByteSize;

    use crate::errors::ParserResult;
    use crate::scanner::{Lexeme, Scanner, ScannerMode};
    use crate::{reader_from_bytes, reader_from_file, reader_from_relative_file};

    #[test]
    fn should_handle_empty_input() {
        let reader = reader_from_bytes!("");
        let scanner = Scanner::new(reader);
        let eoi = scanner
            .with_mode(ScannerMode::IgnoreWhitespace)
            .consume()
            .unwrap();
        assert_eq!(eoi.0, Lexeme::EndOfInput);
    }

    #[test]
    fn should_handle_general_chars() {
        let reader = reader_from_bytes!("{   } [  ]+  - : ,   ");
        let scanner = Scanner::new(reader);
        let mut lexemes: Vec<Lexeme> = vec![];

        while let Ok(lex) = scanner.with_mode(ScannerMode::IgnoreWhitespace).consume() {
            lexemes.push(lex.0);
            if lex.0 == Lexeme::EndOfInput {
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
        let reader = reader_from_bytes!("123456789");
        let scanner = Scanner::new(reader);
        for index in 1..=4 {
            _ = scanner.lookahead(index)
        }
        assert_eq!(scanner.back_coords().column, 4);
        let lex = scanner.consume().unwrap();
        assert_eq!(lex.1.column, 1);
    }

    #[test]
    fn should_handle_whitespace_chars() {
        let reader = reader_from_bytes!(" {  }   \n[]+-:,   ");
        let scanner = Scanner::new(reader);
        let mut lexemes: Vec<Lexeme> = vec![];

        while let Ok(lex) = scanner.with_mode(ScannerMode::ProduceWhitespace).consume() {
            lexemes.push(lex.0);
            if lex.0 == Lexeme::EndOfInput {
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
        let reader = reader_from_bytes!("\\\"\' \t");
        let scanner = Scanner::new(reader);
        let mut lexemes: Vec<Lexeme> = vec![];
        while let Ok(lex) = scanner.with_mode(ScannerMode::ProduceWhitespace).consume() {
            lexemes.push(lex.0);
            if lex.0 == Lexeme::EndOfInput {
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
    #[test]
    fn lookahead_bounds_check() {
        let reader = reader_from_bytes!("{}[],:");
        let scanner = Scanner::new(reader);
        assert!(scanner
            .with_mode(ScannerMode::IgnoreWhitespace)
            .lookahead(34)
            .is_err());
        let _ = scanner
            .with_mode(ScannerMode::IgnoreWhitespace)
            .lookahead(0);
    }

    #[test]
    fn should_scan_basic_test_files_without_panic() {
        for f in fs::read_dir("fixtures/json/valid").unwrap() {
            let path = f.unwrap().path();
            if path.is_file() {
                let start = Instant::now();
                let len = fs::metadata(&path).unwrap().len();
                let reader = reader_from_file!(&path);
                let scanner = Scanner::new(reader);
                loop {
                    let consumed = scanner.with_mode(ScannerMode::ProduceWhitespace).consume();
                    match consumed {
                        Ok(packed) => {
                            if packed.0 == Lexeme::EndOfInput {
                                println!(
                                    "Scanned {} in {:?} [{:?}]",
                                    ByteSize(len),
                                    start.elapsed(),
                                    &path,
                                );
                                break;
                            }
                        }
                        Err(err) => {
                            println!("Error whilst scanning {:?}", &path);
                            panic!()
                        }
                    }
                }
            }
        }
    }
}
