//! General error types for the parser

use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use crate::coords::Coords;
use crate::lexer::Token;

/// Global result type used throughout the parser stages
pub type ParserResult<T> = Result<T, ParserError>;

/// Enumeration of the various different parser stages that can produce an error
#[derive(Debug, Copy, Clone)]
pub enum ParserErrorSource {
    /// The lexer stage of the parser
    Lexer,
    /// The parsing/AST construction stage of the parser
    Parser,
}

impl Display for ParserErrorSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErrorSource::Lexer => write!(f, "lexing"),
            ParserErrorSource::Parser => write!(f, "parsing"),
        }
    }
}

/// A global enumeration of error codes
#[derive(Debug, Clone, PartialEq)]
pub enum ParserErrorDetails {
    InvalidFile,
    ZeroLengthInput,
    EndOfInput,
    StreamFailure,
    NonUtf8InputDetected,
    UnexpectedToken(Token),
    PairExpected,
    InvalidRootObject,
    InvalidObject,
    InvalidArray,
    InvalidCharacter(char),
    MatchFailed(String, String),
    InvalidNumericRepresentation(String),
    InvalidEscapeSequence(String),
    InvalidUnicodeEscapeSequence(String),
}

impl Display for ParserErrorDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErrorDetails::InvalidFile => write!(f, "invalid file specified"),
            ParserErrorDetails::ZeroLengthInput => write!(f, "zero length input"),
            ParserErrorDetails::EndOfInput => write!(f, "end of input reached"),
            ParserErrorDetails::StreamFailure => write!(f, "failure in the underlying stream"),
            ParserErrorDetails::NonUtf8InputDetected => write!(f, "non-UTF8 input"),
            ParserErrorDetails::UnexpectedToken(token) => {
                write!(f, "unexpected token found: {}", token)
            }
            ParserErrorDetails::PairExpected => {
                write!(f, "pair expected, something else was found")
            }
            ParserErrorDetails::InvalidRootObject => write!(f, "invalid JSON"),
            ParserErrorDetails::InvalidObject => write!(f, "invalid object"),
            ParserErrorDetails::InvalidArray => write!(f, "invalid array"),
            ParserErrorDetails::InvalidCharacter(ch) => write!(f, "invalid character: \'{}\'", ch),
            ParserErrorDetails::MatchFailed(first, second) => write!(
                f,
                "a match failed. Looking for \"{}\", found \"{}\"",
                first, second
            ),
            ParserErrorDetails::InvalidNumericRepresentation(repr) => {
                write!(f, "invalid number representation: \"{}\"", repr)
            }
            ParserErrorDetails::InvalidEscapeSequence(seq) => {
                write!(f, "invalid escape sequence: \"{}\"", seq)
            }
            ParserErrorDetails::InvalidUnicodeEscapeSequence(seq) => {
                write!(f, "invalid unicode escape sequence: \"{}\"", seq)
            }
        }
    }
}

/// The general error structure
#[derive(Debug, Clone)]
pub struct ParserError {
    /// The originating source for the error
    pub source: ParserErrorSource,
    /// The global error code for the error
    pub details: ParserErrorDetails,
    /// Parser [Coords]
    pub coords: Coords,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Source: {}, Details: {}, Coords: {}",
            self.source, self.details, self.coords
        )
    }
}

#[macro_export]
macro_rules! lexer_error {
    ($details: expr, $coords : expr) => {
        Err(ParserError {
            source: ParserErrorSource::Lexer,
            details: $details,
            coords: $coords,
        })
    };
}

#[macro_export]
macro_rules! parser_error {
    ($details: expr, $coords: expr) => {
        Err(ParserError {
            source: ParserErrorSource::Parser,
            details: $details,
            coords: $coords,
        })
    };
}
