#![allow(unused_macros)]

use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;

use chisel_stringtable::common::StringTable;

use crate::parser_coords::ParserCoords;
use crate::parser_errors::ParserResult;
use crate::parser_errors::*;
use crate::scanner::{Lexeme, PackedLexeme, Scanner, ScannerMode};
use crate::{is_digit, is_period, lexer_error, unpack_digit};

/// Sequence of literal characters forming a 'null' token
const NULL_SEQUENCE: &[Lexeme] = &[
    Lexeme::Alphabetic('n'),
    Lexeme::Alphabetic('u'),
    Lexeme::Alphabetic('l'),
    Lexeme::Alphabetic('l'),
];
/// Sequence of literal characters forming a 'true' token
const TRUE_SEQUENCE: &[Lexeme] = &[
    Lexeme::Alphabetic('t'),
    Lexeme::Alphabetic('r'),
    Lexeme::Alphabetic('u'),
    Lexeme::Alphabetic('e'),
];
/// Sequence of literal characters forming a 'false' token
const FALSE_SEQUENCE: &[Lexeme] = &[
    Lexeme::Alphabetic('f'),
    Lexeme::Alphabetic('a'),
    Lexeme::Alphabetic('l'),
    Lexeme::Alphabetic('s'),
    Lexeme::Alphabetic('e'),
];

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

#[derive(Debug, Clone)]
pub struct PackedToken {
    /// The actual [Token]
    pub token: Token,
    /// The starting point in the input for the token
    pub start: ParserCoords,
    /// The end point in the input for the token
    pub end: Option<ParserCoords>,
}

/// Convenience macro for packing tokens along with their positional information
macro_rules! packed_token {
    ($t:expr, $s:expr, $e:expr) => {
        PackedToken {
            token: $t,
            start: $s,
            end: Some($e),
        }
    };
    ($t:expr, $s:expr) => {
        PackedToken {
            token: $t,
            start: $s,
            end: None,
        }
    };
}

/// A lexer implementation which will consume a stream of lexemes from a [Scanner] and produce
/// a stream of [Token]s.
#[derive()]
pub struct Lexer<'a, Reader: Debug + Read> {
    /// [StringTable] used for interning all parsed strings
    strings: Rc<RefCell<dyn StringTable<'a, u64>>>,
    /// The [Scanner] instance used by the lexer to source [Lexeme]s
    scanner: Scanner<Reader>,
    /// Internal buffer for hoovering up strings from the input
    buffer: String,
}

impl<'a, Reader: Debug + Read> Lexer<'a, Reader> {
    /// Construct a new [Lexer] instance which will utilise a given [StringTable]
    pub fn new(string_table: Rc<RefCell<dyn StringTable<'a, u64>>>, reader: Reader) -> Self {
        Lexer {
            strings: string_table,
            scanner: Scanner::new(reader),
            buffer: String::new(),
        }
    }

    /// Consume the next token from the input stream. This is a simple LA(1) algorithm,
    /// which looks ahead in the input 1 lexeme, and then based on the grammar rules, attempts
    /// to consume a token based on the prefix found. The working assumption is that ws is skipped
    /// unless parsing out specific types of tokens such as strings, numbers etc...
    pub fn consume(&mut self) -> ParserResult<PackedToken> {
        match self
            .scanner
            .with_mode(ScannerMode::IgnoreWhitespace)
            .lookahead(1)
        {
            Ok(packed) => match packed.lexeme {
                Lexeme::LeftBrace => self.match_start_object(),
                Lexeme::RightBrace => self.match_end_object(),
                Lexeme::LeftBracket => self.match_start_array(),
                Lexeme::RightBracket => self.match_end_array(),
                Lexeme::Comma => self.match_comma(),
                Lexeme::Colon => self.match_colon(),
                Lexeme::Alphabetic(c) => match c {
                    'n' => self.match_null(),
                    't' | 'f' => self.match_bool(c),
                    c => lexer_error!(
                        ParserErrorCode::InvalidCharacter,
                        format!("invalid character found: '{}'", c),
                        self.scanner.back_coords()
                    ),
                },
                Lexeme::Minus => self.match_number('-'),
                Lexeme::Digit(d) => self.match_number(d),
                Lexeme::DoubleQuote => self.match_string(),
                Lexeme::EndOfInput => {
                    Ok(packed_token!(Token::EndOfInput, self.scanner.back_coords()))
                }
                unknown => {
                    lexer_error!(
                        ParserErrorCode::InvalidLexeme,
                        format!("invalid lexeme found: {}", unknown),
                        self.scanner.back_coords()
                    )
                }
            },
            Err(err) => match err.code {
                ParserErrorCode::EndOfInput => {
                    Ok(packed_token!(Token::EndOfInput, self.scanner.back_coords()))
                }
                _ => {
                    lexer_error!(
                        ParserErrorCode::ScannerFailure,
                        "lookahead failed",
                        self.scanner.back_coords(),
                        err
                    )
                }
            },
        }
    }

    /// Consume and match (exactly) a sequence of alphabetic characters from the input stream, returning
    /// the start and end input coordinates if successful
    fn match_exact(&self, seq: &[Lexeme]) -> ParserResult<(ParserCoords, ParserCoords)> {
        for (index, c) in seq.iter().enumerate() {
            match self.scanner.lookahead(index + 1) {
                Ok(packed) => {
                    if packed.lexeme != *c {
                        return lexer_error!(
                            ParserErrorCode::InvalidLexeme,
                            format!("was looking for {}, found {}", c, packed.lexeme),
                            self.scanner.back_coords()
                        );
                    }
                }
                Err(err) => {
                    return lexer_error!(
                        ParserErrorCode::ScannerFailure,
                        "lookahead failed",
                        self.scanner.back_coords(),
                        err
                    );
                }
            }
        }
        Ok((self.scanner.front_coords(), self.scanner.back_coords()))
    }

    /// Attempt to match exactly one of the supplied sequence of [Lexeme]s.  Returns the first
    /// [Lexeme] that matches.
    fn match_one_of(&self, seq: &[Lexeme]) -> ParserResult<PackedLexeme> {
        match self.scanner.lookahead(1) {
            Ok(packed) => {
                if seq.contains(&packed.lexeme) {
                    Ok(packed)
                } else {
                    lexer_error!(
                        ParserErrorCode::MatchFailed,
                        format!("failed to match one of {:?}", seq),
                        self.scanner.back_coords()
                    )
                }
            }
            Err(err) => {
                lexer_error!(
                    ParserErrorCode::ScannerFailure,
                    "lookahead failed",
                    self.scanner.back_coords(),
                    err
                )
            }
        }
    }

    /// Check that a numeric prefix (either '-' or '0') is a valid JSON numeric prefix.
    /// We do this separately prior to attempting any full parse of a numeric given that
    /// fast_float will allow for multiple leading zeros
    fn match_valid_number_prefix(&self, first: char) -> ParserResult<()> {
        let la = self.scanner.lookahead(1)?;
        match first {
            '-' => {
                if is_digit!(la.lexeme) {
                    Ok(())
                } else {
                    lexer_error!(
                        ParserErrorCode::MatchFailed,
                        "invalid numeric prefix found",
                        la.coords
                    )
                }
            }
            '0' => {
                if !is_period!(la.lexeme) {
                    lexer_error!(
                        ParserErrorCode::MatchFailed,
                        "invalid numeric prefix found",
                        la.coords
                    )
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    /// Attempt to match on a number representation.  Utilise the excellent lexical lib in order
    /// to carry out the actual parsing of the numeric value
    fn match_number(&mut self, first: char) -> ParserResult<PackedToken> {
        self.buffer.clear();
        self.buffer.push(first);
        let start_coords = self
            .scanner
            .with_mode(ScannerMode::ProduceWhitespace)
            .consume()?
            .coords;

        self.match_valid_number_prefix(first)?;
        loop {
            let packed = self.scanner.consume()?;
            match packed.lexeme {
                Lexeme::Period => self.buffer.push('.'),
                Lexeme::Digit(d) => self.buffer.push(d),
                Lexeme::Minus => self.buffer.push('-'),
                Lexeme::Plus => self.buffer.push('+'),
                Lexeme::Alphabetic(c) => match c {
                    'e' | 'E' => self.buffer.push(c),
                    _ => {
                        return lexer_error!(
                            ParserErrorCode::EndOfInput,
                            "invalid character found whilst parsing number",
                            packed.coords
                        );
                    }
                },
                Lexeme::NewLine | Lexeme::Comma => break,
                Lexeme::Whitespace(_) => break,
                Lexeme::EndOfInput => {
                    return lexer_error!(
                        ParserErrorCode::EndOfInput,
                        "end of input found whilst parsing string",
                        packed.coords
                    );
                }
                _ => break,
            }
        }

        match fast_float::parse(&self.buffer) {
            Ok(n) => Ok(packed_token!(
                Token::Num(n),
                start_coords,
                self.scanner.back_coords()
            )),
            Err(_) => lexer_error!(
                ParserErrorCode::MatchFailed,
                "invalid number found in input",
                start_coords
            ),
        }
    }

    /// Attempts to match a string token, including any escaped characters.  Does *not* perform
    /// any translation of escaped characters so that the token internals are capture in their
    /// original format
    fn match_string(&mut self) -> ParserResult<PackedToken> {
        self.buffer.clear();
        self.buffer.push('\"');

        let start_coords = self
            .scanner
            .with_mode(ScannerMode::ProduceWhitespace)
            .consume()?
            .coords;

        loop {
            let packed = self.scanner.consume()?;
            match packed.lexeme {
                Lexeme::Escape => self.match_escape_sequence()?,
                Lexeme::DoubleQuote => {
                    self.buffer.push('\"');
                    break;
                }
                Lexeme::NonAlphabetic(c) => self.buffer.push(c),
                Lexeme::Alphabetic(c) => self.buffer.push(c),
                Lexeme::Digit(c) => self.buffer.push(c),
                Lexeme::Whitespace(c) => self.buffer.push(c),
                Lexeme::Plus => self.buffer.push('+'),
                Lexeme::Minus => self.buffer.push('-'),
                Lexeme::Colon => self.buffer.push(':'),
                Lexeme::Comma => self.buffer.push(':'),
                Lexeme::Period => self.buffer.push('.'),
                Lexeme::EndOfInput => {
                    return lexer_error!(
                        ParserErrorCode::EndOfInput,
                        "end of input found whilst parsing string",
                        packed.coords
                    );
                }
                Lexeme::NewLine => {
                    return lexer_error!(
                        ParserErrorCode::EndOfInput,
                        "newline found whilst parsing string",
                        packed.coords
                    );
                }
                _ => break,
            }
        }

        let mut strings = self.strings.borrow_mut();
        if let Some(hash) = strings.contains(self.buffer.as_str()) {
            Ok(packed_token!(
                Token::Str(hash),
                start_coords,
                self.scanner.back_coords()
            ))
        } else {
            Ok(packed_token!(
                Token::Str(strings.add(self.buffer.as_str())),
                start_coords,
                self.scanner.back_coords()
            ))
        }
    }

    /// Match a valid string escape sequence
    fn match_escape_sequence(&mut self) -> ParserResult<()> {
        self.buffer.push('\\');
        let packed = self.scanner.consume()?;
        match packed.lexeme {
            Lexeme::DoubleQuote => self.buffer.push('\"'),
            Lexeme::Alphabetic(c) => match c {
                'u' => {
                    self.buffer.push(c);
                    self.match_unicode_escape_sequence()?
                }
                'n' | 't' | 'r' | '\\' | '/' | 'b' | 'f' => self.buffer.push(c),
                _ => {
                    return lexer_error!(
                        ParserErrorCode::InvalidCharacter,
                        "invalid escape sequence detected",
                        packed.coords
                    );
                }
            },
            _ => (),
        }
        Ok(())
    }

    /// Match a valid unicode escape sequence in the form uXXXX where each X is a valid hex
    /// digit
    fn match_unicode_escape_sequence(&mut self) -> ParserResult<()> {
        for _ in 1..=4 {
            let packed = self.scanner.consume()?;
            match packed.lexeme {
                Lexeme::Alphabetic(c) | Lexeme::Digit(c) => {
                    if c.is_ascii_hexdigit() {
                        self.buffer.push(c);
                    } else {
                        return lexer_error!(
                            ParserErrorCode::InvalidCharacter,
                            "invalid hex escape code detected",
                            packed.coords
                        );
                    }
                }
                _ => {
                    return lexer_error!(
                        ParserErrorCode::InvalidCharacter,
                        "invalid escape sequence detected",
                        packed.coords
                    );
                }
            }
        }
        Ok(())
    }

    /// Consume a nulll token from the input and and return a [PackedToken]
    fn match_null(&self) -> ParserResult<PackedToken> {
        match self.match_exact(NULL_SEQUENCE) {
            Ok((start, end)) => {
                self.scanner.discard(4);
                Ok(packed_token!(Token::Null, start, end))
            }
            Err(_) => lexer_error!(
                ParserErrorCode::MatchFailed,
                "expected null, couldn't match",
                self.scanner.back_coords()
            ),
        }
    }

    /// Consume a bool token from the input and return a [PackedToken]
    fn match_bool(&self, prefix: char) -> ParserResult<PackedToken> {
        match prefix {
            't' => match self.match_exact(TRUE_SEQUENCE) {
                Ok((start, end)) => {
                    self.scanner.discard(4);
                    Ok(packed_token!(Token::Bool(true), start, end))
                }
                Err(err) => lexer_error!(
                    ParserErrorCode::MatchFailed,
                    format!("failed to parse a bool"),
                    self.scanner.back_coords(),
                    err
                ),
            },
            'f' => match self.match_exact(FALSE_SEQUENCE) {
                Ok((start, end)) => {
                    self.scanner.discard(5);
                    Ok(packed_token!(Token::Bool(false), start, end))
                }
                Err(err) => lexer_error!(
                    ParserErrorCode::MatchFailed,
                    format!("failed to parse a bool"),
                    self.scanner.back_coords(),
                    err
                ),
            },
            _ => panic!(),
        }
    }

    /// Consume a left brace from the input and and return a [PackedToken]
    fn match_start_object(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::StartObject, result.coords))
    }

    /// Consume a right brace from the input and and return a [PackedToken]
    fn match_end_object(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::EndObject, result.coords))
    }

    /// Consume a left bracket from the input and and return a [PackedToken]
    fn match_start_array(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::StartArray, result.coords))
    }

    /// Consume a right bracket from the input and and return a [PackedToken]
    fn match_end_array(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::EndArray, result.coords))
    }

    /// Consume a comma from the input and and return a [PackedToken]
    fn match_comma(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::Comma, result.coords))
    }

    /// Consume a colon from the input and and return a [PackedToken]
    fn match_colon(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::Colon, result.coords))
    }
}
