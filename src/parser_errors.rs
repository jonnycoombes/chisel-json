//! General error types for the parser

use std::borrow::Cow;

use crate::parser_coords::ParserCoords;

/// Global result type used throughout the parser stages
pub type ParserResult<T> = Result<T, ParserError>;

/// Enumeration of the various different parser stages that can produce an error
#[derive(Debug, Copy, Clone)]
pub enum ParserStage {
    /// The stream stage of the parser
    Stream,
    /// The scanning stage of the parser
    Scanner,
    /// The lexer stage of the parser
    Lexer,
    /// The parsing/AST construction stage of the parser
    Parser,
}

/// A global enumeration of error codes
#[derive(Debug, Copy, Clone)]
pub enum ParserErrorCode {
    EndOfInput,
    IOError,
    InvalidByteSequence,
    InvalidCharSequence,
    InvalidCharacter,
    InvalidLexeme,
    MatchFailed,
    ExpectedLexeme,
    ScannerFailure,
    StreamFailure,
}

/// The general error structure
#[derive(Debug, Clone)]
pub struct ParserError {
    /// The originating stage for the error
    pub stage: ParserStage,
    /// The global error code for the error
    pub code: ParserErrorCode,
    /// Additional information about the error
    pub message: Cow<'static, str>,
    /// Optional parser coordinates
    pub coords: Option<ParserCoords>,
    /// An optional inner error
    pub inner: Option<Box<ParserError>>,
}

/// Produce a stream specific error
#[macro_export]
macro_rules! stream_error {
    ($code: expr, $msg : expr) => {
        Err(ParserError {
            stage: ParserStage::Stream,
            code: $code,
            message: $msg.into(),
            coords: None,
            inner: None,
        })
    };
}

#[macro_export]
macro_rules! scanner_error {
    ($code: expr, $msg : expr) => {
        Err(ParserError {
            stage: ParserStage::Scanner,
            code: $code,
            message: $msg.into(),
            coords: None,
            inner: None,
        })
    };
    ($code: expr, $msg : expr, $coords : expr) => {
        Err(ParserError {
            stage: ParserStage::Scanner,
            code: $code,
            message: $msg.into(),
            coords: Some($coords),
            inner: None,
        })
    };
    ($code: expr, $msg : expr, $coords : expr, $inner : expr) => {
        Err(ParserError {
            stage: ParserStage::Scanner,
            code: $code,
            message: $msg.into(),
            coords: Some($coords),
            inner: Some(Box::new($inner.clone())),
        })
    };
}

#[macro_export]
macro_rules! lexer_error {
    ($code: expr, $msg : expr) => {
        Err(ParserError {
            stage: ParserStage::Lexer,
            code: $code,
            message: $msg.into(),
            coords: None,
            inner: None,
        })
    };
    ($code: expr, $msg : expr, $coords : expr) => {
        Err(ParserError {
            stage: ParserStage::Lexer,
            code: $code,
            message: $msg.into(),
            coords: Some($coords),
            inner: None,
        })
    };
    ($code: expr, $msg : expr, $coords : expr, $inner : expr) => {
        Err(ParserError {
            stage: ParserStage::Lexer,
            code: $code,
            message: $msg.into(),
            coords: Some($coords),
            inner: Some(Box::new($inner.clone())),
        })
    };
}
