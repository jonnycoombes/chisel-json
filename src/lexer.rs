use crate::coords::{Coords, Span};
use crate::errors::{ParserError, ParserErrorCode, ParserResult, ParserStage};
use crate::parser::Parser;
use crate::{lexer_error, parser_error};
use chisel_decoders::common::{DecoderError, DecoderErrorCode, DecoderResult};
use chisel_decoders::utf8::Utf8Decoder;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::io::BufRead;
use std::rc::Rc;

/// Default lookahead buffer size
const DEFAULT_BUFFER_SIZE: usize = 4096;
/// Pattern to match for null
const NULL_PATTERN: [char; 4] = ['n', 'u', 'l', 'l'];
/// Pattern to match for true
const TRUE_PATTERN: [char; 4] = ['t', 'r', 'u', 'e'];
/// Pattern to match for false
const FALSE_PATTERN: [char; 5] = ['f', 'a', 'l', 's', 'e'];

/// Enumeration of valid JSON tokens
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    StartObject,
    EndObject,
    StartArray,
    EndArray,
    Colon,
    Comma,
    Str(String),
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
        Ok(($t, Span { start: $s, end: $e }))
    };
    ($t:expr, $s:expr) => {
        Ok(($t, Span { start: $s, end: $s }))
    };
}

macro_rules! match_digits {
    () => {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
    }
}

macro_rules! match_non_zero_digits {
    () => {
        '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
    }
}

pub struct Lexer<B: BufRead> {
    /// The input [Utf8Decoder]
    decoder: Utf8Decoder<B>,

    /// Lookahead buffer
    buffer: Vec<char>,

    /// Optional pushback character
    pushback: Option<char>,

    /// Current input [Coords]
    coords: Coords,
}

impl<B: BufRead> Lexer<B> {
    pub fn new(reader: B) -> Self {
        Lexer {
            decoder: Utf8Decoder::new(reader),
            buffer: Vec::with_capacity(DEFAULT_BUFFER_SIZE),
            pushback: None,
            coords: Coords::default(),
        }
    }

    /// Reset the current state
    fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Consume the next [Token] from the input
    pub fn consume(&mut self) -> ParserResult<PackedToken> {
        self.reset();
        match self.advance(true) {
            Ok(_) => match self.buffer[0] {
                '{' => packed_token!(Token::StartObject, self.coords),
                '}' => packed_token!(Token::EndObject, self.coords),
                '[' => packed_token!(Token::StartArray, self.coords),
                ']' => packed_token!(Token::EndArray, self.coords),
                ':' => packed_token!(Token::Colon, self.coords),
                ',' => packed_token!(Token::Comma, self.coords),
                '\"' => self.match_string(),
                'n' => self.match_null(),
                't' => self.match_true(),
                'f' => self.match_false(),
                '-' => self.match_number(),
                d if d.is_ascii_digit() => self.match_number(),
                ch => lexer_error!(
                    ParserErrorCode::InvalidCharacter,
                    format!("invalid character found in input: \"{}\"", ch),
                    self.coords
                ),
            },
            Err(err) => match err.code {
                ParserErrorCode::EndOfInput => {
                    packed_token!(Token::EndOfInput, self.coords)
                }
                _ => lexer_error!(err.code, err.message, err.coords.unwrap()),
            },
        }
    }

    /// Match on a valid Json string.
    fn match_string(&mut self) -> ParserResult<PackedToken> {
        let start_coords = self.coords;
        loop {
            match self.advance(false) {
                Ok(_) => match self.buffer.last().unwrap() {
                    '\\' => match self.advance(false) {
                        Ok(_) => match self.buffer.last().unwrap() {
                            'n' | 't' | 'r' | '\\' | '/' | 'b' | 'f' | '\"' => (),
                            'u' => match self.advance_n(4, false) {
                                Ok(_) => {
                                    for i in 1..=4 {
                                        if !self.buffer[self.buffer.len() - i].is_ascii_hexdigit() {
                                            return lexer_error!(
                                                ParserErrorCode::InvalidUnicodeEscapeSequence,
                                                format!("invalid unicode escape sequence detected"),
                                                self.coords
                                            );
                                        }
                                    }
                                }
                                Err(err) => {
                                    return lexer_error!(err.code, err.message, err.coords.unwrap());
                                }
                            },
                            ch => {
                                return lexer_error!(
                                    ParserErrorCode::InvalidEscapeSequence,
                                    format!("found illegal escape sequence: \"\\{}\"", ch),
                                    self.coords
                                );
                            }
                        },
                        Err(err) => {
                            return lexer_error!(err.code, err.message, err.coords.unwrap());
                        }
                    },
                    '\"' => {
                        return packed_token!(Token::Str(self.buffer_to_string()), start_coords, self.coords);
                    }
                    _ => (),
                },
                Err(err) => return lexer_error!(err.code, err.message, err.coords.unwrap()),
            }
        }
    }

    /// Match on a valid Json number representation, taking into account valid prefixes allowed
    /// within Json but discarding anything that may be allowed by a more general representations.
    ///
    /// Few rules are applied here, leading to different error conditions:
    /// - All representations must have a valid prefix
    /// - Only a single exponent can be specified
    /// - Only a single decimal point can be specified
    /// - Exponents must be well-formed
    /// - An non-exponent alphabetic found in the representation will result in an error
    /// - Numbers can be terminated by commas, brackets and whitespace only (end of pair, end of array)
    fn match_number(&mut self) -> ParserResult<PackedToken> {
        let start_coords = self.coords;
        let mut have_exponent = false;
        let mut have_decimal = false;
        match self.match_valid_number_prefix() {
            Ok(()) => loop {
                match self.advance(false) {
                    Ok(_) => match self.buffer.last().unwrap() {
                        match_digits!() => (),
                        'e' | 'E' => {
                            if !have_exponent {
                                match self.advance(false) {
                                    Ok(_) => match self.buffer.last().unwrap() {
                                        '+' | '-' => (),
                                        _ => {
                                            return lexer_error!(
                                                ParserErrorCode::InvalidNumericRepresentation,
                                                "malformed exponent detected",
                                                self.coords
                                            );
                                        }
                                    },
                                    Err(err) => {
                                        return lexer_error!(
                                            err.code,
                                            err.message,
                                            err.coords.unwrap()
                                        );
                                    }
                                }
                                have_exponent = true;
                            } else {
                                return lexer_error!(
                                    ParserErrorCode::InvalidNumericRepresentation,
                                    "found multiple exponents",
                                    self.coords
                                );
                            }
                        }
                        '.' => {
                            if !have_decimal {
                                have_decimal = true;
                            } else {
                                return lexer_error!(
                                    ParserErrorCode::InvalidNumericRepresentation,
                                    "found multiple decimal points",
                                    self.coords
                                );
                            }
                        }
                        ch if ch.is_alphabetic() => {
                            return lexer_error!(
                                ParserErrorCode::InvalidNumericRepresentation,
                                "found non-numerics within representation",
                                self.coords
                            );
                        }
                        ']' | ',' | '}' => {
                            self.pushback();
                            break;
                        }
                        ch if ch.is_whitespace() => {
                            self.pushback();
                            break;
                        }
                        ch => {
                            return lexer_error!(
                                ParserErrorCode::InvalidNumericRepresentation,
                                format!(
                                    "found an invalid character terminating a number: \"{}\"",
                                    ch
                                ),
                                self.coords
                            );
                        }
                    },
                    Err(err) => return lexer_error!(err.code, err.message, err.coords.unwrap()),
                }
            },
            Err(err) => {
                return lexer_error!(err.code, err.message, err.coords.unwrap());
            }
        }

        self.try_parse_buffer_to_float(start_coords, self.coords)
    }

    /// Convert the contents of the buffer into an owned [String]
    #[inline]
    fn buffer_to_string(&self) -> String {
        self.buffer.iter().collect()
    }

    /// Use the fast float library to try and parse out an [f64] from the current buffer contents
    #[inline]
    fn try_parse_buffer_to_float(
        &mut self,
        start_coords: Coords,
        end_coords: Coords,
    ) -> ParserResult<PackedToken> {
        match fast_float::parse(self.buffer_to_string()) {
            Ok(n) => packed_token!(Token::Num(n), start_coords, end_coords),
            Err(_) => lexer_error!(
                ParserErrorCode::MatchFailed,
                "invalid number found in input",
                start_coords
            ),
        }
    }

    /// Check that a numeric representation is prefixed correctly.
    ///
    /// A few rules here:
    /// - A leading minus must be followed by a digit
    /// - A leading minus must be followed by at most one zero before a period
    /// - Any number > zero can't have a leading zero in the representation
    fn match_valid_number_prefix(&mut self) -> ParserResult<()> {
        assert!(self.buffer[0].is_ascii_digit() || self.buffer[0] == '-');
        match self.buffer[0] {
            '-' => {
                self.advance(false)
                    .and_then(|_| self.check_following_minus())
            }
            '0' => {
                self.advance(false)
                    .and_then(|_| self.check_following_zero())
            }
            _ => Ok(()),
        }
    }

    #[inline]
    fn check_following_zero(&mut self) -> Result<(), ParserError> {
        match self.buffer[1] {
            '.' => Ok(()),
            match_digits!() =>
                lexer_error!(ParserErrorCode::InvalidNumericRepresentation,
                                    "only one leading zero is allowed", self.coords),
            _ => {
                self.pushback();
                Ok(())
            }
        }
    }

    #[inline]
    fn check_following_minus(&mut self) -> Result<(), ParserError> {
        match self.buffer[1] {
            match_non_zero_digits!() => Ok(()),
            '0' => {
                self.advance(false)
                    .and_then(|_| {
                        if self.buffer[2] != '.' {
                            return lexer_error!(
                                    ParserErrorCode::InvalidNumericRepresentation,
                                    "only one leading zero is allowed",
                                    self.coords
                                );
                        }
                        Ok(())
                    })
            }
            ch =>
                lexer_error!(ParserErrorCode::InvalidNumericRepresentation,
                                format!("minus followed by illegal character: \"{}\"", ch), self.coords)
        }
    }

    /// Match on a null token
    fn match_null(&mut self) -> ParserResult<PackedToken> {
        let start_coords = self.coords;
        self.advance_n(3, false)
            .and_then(|_| {
                if self.buffer[0..=3] == NULL_PATTERN {
                    packed_token!(Token::Null, start_coords, self.coords)
                } else {
                    lexer_error!(
                        ParserErrorCode::MatchFailed,
                        "\"null\" expected",
                        start_coords
                    )
                }
            })
    }

    /// Match on a true token
    fn match_true(&mut self) -> ParserResult<PackedToken> {
        let start_coords = self.coords;
        self.advance_n(3, false)
            .and_then(|_| {
                if self.buffer[0..=3] == TRUE_PATTERN {
                    packed_token!(Token::Bool(true), start_coords, self.coords)
                } else {
                    lexer_error!(
                        ParserErrorCode::MatchFailed,
                        "\"true\" expected",
                        start_coords
                    )
                }
            })
    }

    /// Match on a false token
    fn match_false(&mut self) -> ParserResult<PackedToken> {
        let start_coords = self.coords;
        self.advance_n(4, false)
            .and_then(|_| {
                if self.buffer[0..=4] == FALSE_PATTERN {
                    packed_token!(Token::Bool(false), start_coords, self.coords)
                } else {
                    lexer_error!(
                        ParserErrorCode::MatchFailed,
                        "\"null\" expected",
                        start_coords
                    )
                }
            })
    }

    /// Get the next character from either the pushback or from the decoder
    #[inline]
    fn next_char(&mut self) -> DecoderResult<char> {
        match self.pushback {
            Some(c) => {
                self.pushback = None;
                Ok(c)
            }
            None => self.decoder.decode_next(),
        }
    }

    /// Transfer the last character in the buffer to the pushback
    #[inline]
    fn pushback(&mut self) {
        if !self.buffer.is_empty() {
            self.pushback = self.buffer.pop();
            self.coords.absolute -= 1;
            self.coords.column -= 1;
        } else {
            self.pushback = None;
        }
    }

    /// Advance n characters in the input
    #[inline]
    fn advance_n(&mut self, n: usize, skip_whitespace: bool) -> ParserResult<()> {
        for _ in 0..n {
            match self.advance(skip_whitespace) {
                Ok(_) => (),
                Err(err) => return lexer_error!(err.code, err.message, err.coords.unwrap()),
            }
        }
        Ok(())
    }

    /// Advance a character in the input stream, and push onto the end of the internal buffer. This
    /// will update the current input [Coords]. Optionally skip whitespace in the input, (but still
    /// update the coordinates accordingly).
    fn advance(&mut self, skip_whitespace: bool) -> ParserResult<()> {
        loop {
            match self.next_char() {
                Ok(c) => {
                    self.coords.inc(c == '\n');
                    if skip_whitespace {
                        if !c.is_whitespace() {
                            self.buffer.push(c);
                            break;
                        }
                    } else {
                        self.buffer.push(c);
                        break;
                    }
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
                    };
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::env;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::rc::Rc;
    use std::time::Instant;

    use crate::coords::{Coords, Span};
    use crate::errors::{ParserError, ParserResult};
    use crate::lexer::{Lexer, PackedToken, Token};
    use crate::{lines_from_relative_file, reader_from_bytes};

    #[test]
    fn should_parse_basic_tokens() {
        let reader = reader_from_bytes!("{}[],:");
        let mut lexer = Lexer::new(reader);
        let mut tokens: Vec<Token> = vec![];
        let mut spans: Vec<Span> = vec![];
        for _ in 1..=7 {
            let token = lexer.consume().unwrap();
            tokens.push(token.0);
            spans.push(token.1);
        }
        assert_eq!(
            tokens,
            [
                Token::StartObject,
                Token::EndObject,
                Token::StartArray,
                Token::EndArray,
                Token::Comma,
                Token::Colon,
                Token::EndOfInput
            ]
        );
    }

    #[test]
    fn should_parse_null_and_booleans() {
        let reader = reader_from_bytes!("null true    falsetruefalse");
        let mut lexer = Lexer::new(reader);
        let mut tokens: Vec<Token> = vec![];
        let mut spans: Vec<Span> = vec![];
        for _ in 1..=6 {
            let token = lexer.consume().unwrap();
            tokens.push(token.0);
            spans.push(token.1);
        }
        assert_eq!(
            tokens,
            [
                Token::Null,
                Token::Bool(true),
                Token::Bool(false),
                Token::Bool(true),
                Token::Bool(false),
                Token::EndOfInput
            ]
        );
    }

    #[test]
    fn should_parse_strings() {
        let lines = lines_from_relative_file!("fixtures/utf-8/strings.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                let reader = reader_from_bytes!(l);
                let mut lexer = Lexer::new(reader);
                let token = lexer.consume().unwrap();
                match token.0 {
                    Token::Str(str) => {
                        assert_eq!(str, l)
                    }
                    _ => panic!(),
                }
            }
        }
    }

    #[test]
    fn should_parse_numerics() {
        let start = Instant::now();
        let lines = lines_from_relative_file!("fixtures/utf-8/numbers.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                let reader = reader_from_bytes!(l);
                let mut lexer = Lexer::new(reader);
                let token = lexer.consume().unwrap();
                assert_eq!(
                    token.0,
                    Token::Num(fast_float::parse(l.replace(',', "")).unwrap())
                );
            }
        }
        println!("Parsed numerics in {:?}", start.elapsed());
    }

    #[test]
    fn should_correctly_handle_invalid_numbers() {
        let lines = lines_from_relative_file!("fixtures/utf-8/invalid_numbers.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                let reader = reader_from_bytes!(l);
                let mut lexer = Lexer::new(reader);
                let token = lexer.consume();
                assert!(token.is_err());
            }
        }
    }

    #[test]
    fn should_correctly_identity_dodgy_strings() {
        let lines = lines_from_relative_file!("fixtures/utf-8/dodgy_strings.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                let reader = reader_from_bytes!(l);
                let mut lexer = Lexer::new(reader);
                let mut error_token: Option<ParserError> = None;
                loop {
                    let token = lexer.consume();
                    match token {
                        Ok(packed) => {
                            if packed.0 == Token::EndOfInput {
                                break;
                            }
                        }
                        Err(err) => {
                            error_token = Some(err.clone());
                            println!(
                                "Dodgy string found: '{}' -> {} : {}",
                                l,
                                err.message,
                                err.coords.unwrap()
                            );
                            break;
                        }
                    }
                }
                assert!(error_token.is_some());
            }
        }
    }

    #[test]
    fn should_correctly_report_errors_for_booleans() {
        let reader = reader_from_bytes!("true farse");
        let mut lexer = Lexer::new(reader);
        let mut results: Vec<ParserResult<PackedToken>> = vec![];
        for _ in 1..=2 {
            results.push(lexer.consume());
        }
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        println!("Parse error: {:?}", results[1]);
    }
}
