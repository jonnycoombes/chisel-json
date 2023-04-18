//! General error types for the parser

use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use crate::coords::Coords;
use crate::lexer::Token;

/// Global result type used throughout the parser stages
pub type ParserResult<T> = Result<T, Error>;

/// Enumeration of the various different parser stages that can produce an error
#[derive(Debug, Copy, Clone)]
pub enum Stage {
    /// The lexer stage of the parser
    Lexer,
    /// The parsing/AST construction stage of the parser
    Parser,
}

impl Display for Stage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Lexer => write!(f, "lexing"),
            Stage::Parser => write!(f, "parsing"),
        }
    }
}

/// A global enumeration of error codes
#[derive(Debug, Clone, PartialEq)]
pub enum Details {
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

impl Display for Details {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Details::EndOfInput => write!(f, "end of input reached"),
            Details::StreamFailure => write!(f, "failure in the underlying stream"),
            Details::NonUtf8InputDetected => write!(f, "non-UTF8 input"),
            Details::UnexpectedToken(token) => write!(f, "unexpected token found: {}", token),
            Details::PairExpected => write!(f, "pair expected, something else was found"),
            Details::InvalidRootObject => write!(f, "invalid JSON"),
            Details::InvalidObject => write!(f, "invalid object"),
            Details::InvalidArray => write!(f, "invalid array"),
            Details::InvalidCharacter(ch) => write!(f, "invalid character: \'{}\'", ch),
            Details::MatchFailed(first, second) => write!(
                f,
                "a match failed. Looking for \"{}\", found \"{}\"",
                first, second
            ),
            Details::InvalidNumericRepresentation(repr) => {
                write!(f, "invalid number representation: \"{}\"", repr)
            }
            Details::InvalidEscapeSequence(seq) => {
                write!(f, "invalid escape sequence: \"{}\"", seq)
            }
            Details::InvalidUnicodeEscapeSequence(seq) => {
                write!(f, "invalid unicode escape sequence: \"{}\"", seq)
            }
        }
    }
}

/// The general error structure
#[derive(Debug, Clone)]
pub struct Error {
    /// The originating stage for the error
    pub stage: Stage,
    /// The global error code for the error
    pub details: Details,
    /// Parser [Coords]
    pub coords: Coords,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stage: {}, Details: {}, Coords: {}",
            self.stage, self.details, self.coords
        )
    }
}

#[macro_export]
macro_rules! lexer_error {
    ($details: expr, $coords : expr) => {
        Err(Error {
            stage: Stage::Lexer,
            details: $details,
            coords: $coords,
        })
    };
}

#[macro_export]
macro_rules! parser_error {
    ($details: expr, $coords: expr) => {
        Err(Error {
            stage: Stage::Parser,
            details: $details,
            coords: $coords,
        })
    };
}
