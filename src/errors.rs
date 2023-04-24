//! Error and [Result] types
//!
//! This module contains definitions for the main [Result] types used throughout the parser.

use crate::coords::Coords;
use crate::lexer::Token;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::io::BufRead;

/// Global result type used throughout the parser stages
pub type ParserResult<T> = Result<T, ParserError>;

/// Enumeration of the various different parser stages that can produce an error
#[derive(Debug, Copy, Clone)]
pub enum ParserErrorSource {
    /// The lexing stage of the parser
    Lexer,
    /// The parsing stage of the DOM parser
    DomParser,
    /// The parsing stage of the SAX parser
    SaxParser,
}

impl Display for ParserErrorSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErrorSource::Lexer => write!(f, "lexing"),
            ParserErrorSource::DomParser => write!(f, "DOM parsing"),
            ParserErrorSource::SaxParser => write!(f, "SAX parsing"),
        }
    }
}

/// A global enumeration of error codes
#[derive(Debug, Clone, PartialEq)]
pub enum ParserErrorDetails {
    /// An invalid file has been specified.  It might not exist, or might not be accessible
    InvalidFile,
    /// We can't parse nothing.
    ZeroLengthInput,
    /// End of input has been reached. This is used as a stopping condition at various points.
    EndOfInput,
    /// If pulling bytes from an underlying stream (or [BufRead]) of some description, and an
    /// error occurs, this will be returned.
    StreamFailure,
    /// Dodgy UTF8 has been found in the input.
    NonUtf8InputDetected,
    /// Edge case error condition. This means that something has gone horribly wrong with the
    /// parse.
    UnexpectedToken(Token),
    /// KV pair is expected but not detected.
    PairExpected,
    /// Supplied JSON doesn't have an object or array as a root object.
    InvalidRootObject,
    /// The parse of an object has failed.
    InvalidObject,
    /// The parse of an array has failed.
    InvalidArray,
    /// An invalid character has been detected within the input.
    InvalidCharacter(char),
    /// Whilst looking for a literal string token (null, true, false) a match couldn't be found
    MatchFailed(String, String),
    /// A number has been found with an incorrect string representation.
    InvalidNumericRepresentation(String),
    /// An invalid escape sequence has been found within the input.
    InvalidEscapeSequence(String),
    /// An invalid unicode escape sequence (\uXXX) has been found within the input.
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
    pub coords: Option<Coords>,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.coords.is_some() {
            write!(
                f,
                "Source: {}, Details: {}, Coords: {}",
                self.source,
                self.details,
                self.coords.unwrap()
            )
        } else {
            write!(f, "Source: {}, Details: {}", self.source, self.details)
        }
    }
}

/// Helper macro for cooking up a [ParserError] specific to the lexer
#[macro_export]
macro_rules! lexer_error {
    ($details: expr, $coords : expr) => {
        Err(ParserError {
            source: ParserErrorSource::Lexer,
            details: $details,
            coords: Some($coords),
        })
    };
    ($details: expr) => {
        Err(ParserError {
            source: ParserErrorSource::Lexer,
            details: $details,
            coords: None,
        })
    };
}

/// Helper macro for cooking up a [ParserError] specific to the DOM parser
#[macro_export]
macro_rules! dom_parser_error {
    ($details: expr, $coords: expr) => {
        Err(ParserError {
            source: ParserErrorSource::DomParser,
            details: $details,
            coords: Some($coords),
        })
    };
    ($details: expr) => {
        Err(ParserError {
            source: ParserErrorSource::DomParser,
            details: $details,
            coords: None,
        })
    };
}

/// Helper macro for cooking up a [ParserError] specific to the SAX parser
#[macro_export]
macro_rules! sax_parser_error {
    ($details: expr, $coords: expr) => {
        Err(ParserError {
            source: ParserErrorSource::SaxParser,
            details: $details,
            coords: Some($coords),
        })
    };
    ($details: expr) => {
        Err(ParserError {
            source: ParserErrorSource::SaxParser,
            details: $details,
            coords: None,
        })
    };
}
