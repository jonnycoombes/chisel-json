//! General error types for the parser

use std::borrow::Cow;

use crate::coords::Coords;

/// Global result type used throughout the parser stages
pub type ParserResult<T> = Result<T, ParserError>;

/// Enumeration of the various different parser stages that can produce an error
#[derive(Debug, Copy, Clone)]
pub enum Stage {
    /// The lexer stage of the parser
    Lexer,
    /// The parsing/AST construction stage of the parser
    Parser,
}

/// A global enumeration of error codes
#[derive(Debug, Clone)]
pub enum Details {
    EndOfInput,
    StreamFailure,
    NonUtf8InputDetected,
    UnexpectedToken,
    PairExpected,
    InvalidObject,
    InvalidArray,
    InvalidCharacter(char),
    MatchFailed,
    InvalidNumericRepresentation(String),
    InvalidEscapeSequence(String),
    InvalidUnicodeEscapeSequence(String),
}

/// The general error structure
#[derive(Debug, Clone)]
pub struct ParserError {
    /// The originating stage for the error
    pub stage: Stage,
    /// The global error code for the error
    pub details: Details,
    /// Optional parser coordinates
    pub coords: Option<Coords>,
}

#[macro_export]
macro_rules! lexer_error {
    ($details: expr, $coords : expr) => {
        Err(ParserError {
            stage: Stage::Lexer,
            details: $details,
            coords: Some($coords),
        })
    };
}

#[macro_export]
macro_rules! parser_error {
    ($details: expr, $coords: expr) => {
        Err(ParserError {
            stage: Stage::Parser,
            details: $details,
            coords: Some($coords),
        })
    };
}
