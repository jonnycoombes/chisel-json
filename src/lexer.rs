#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
use crate::coords::{Coords, Span};
use crate::errors::{Details, Error, ParserResult, Stage};
use crate::parser::DomParser;
use crate::{lexer_error, parser_error};
use chisel_decoders::common::{DecoderError, DecoderErrorCode, DecoderResult};
use chisel_decoders::utf8::Utf8Decoder;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::fmt::{Display, Formatter};
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
    Float(f64),
    Integer(i64),
    Null,
    Bool(bool),
    EndOfInput,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::StartObject => write!(f, "StartObject"),
            Token::EndObject => write!(f, "EndObject"),
            Token::StartArray => write!(f, "StartArray"),
            Token::EndArray => write!(f, "EndArray"),
            Token::Colon => write!(f, "Colon"),
            Token::Comma => write!(f, "Comma"),
            Token::Str(str) => write!(f, "String(\"{}\")", str),
            Token::Float(num) => write!(f, "Num({})", num),
            Token::Integer(num) => write!(f, "Num({})", num),
            Token::Null => write!(f, "Null"),
            Token::Bool(bool) => write!(f, "Bool({})", bool),
            Token::EndOfInput => write!(f, "EndOfInput"),
        }
    }
}

/// A packed token consists of a [Token] and the [Span] associated with it
pub type PackedToken<'a> = (Token, Span);

/// Convenience macro for packing tokens along with their positional information
macro_rules! packed_token {
    ($t:expr, $s:expr, $e:expr) => {
        Ok(($t, Span { start: $s, end: $e }))
    };
    ($t:expr, $s:expr) => {
        Ok(($t, Span { start: $s, end: $s }))
    };
}

macro_rules! match_zero {
    () => {
        '0'
    };
}

macro_rules! match_minus {
    () => {
        '-'
    };
}

macro_rules! match_plus_minus {
    () => {
        '+' | '-'
    };
}

macro_rules! match_digit {
    () => {
        '0'..='9'
    };
}

macro_rules! match_non_zero_digit {
    () => {
        '1'..='9'
    };
}

macro_rules! match_exponent {
    () => {
        'e' | 'E'
    };
}

macro_rules! match_period {
    () => {
        '.'
    };
}

macro_rules! match_numeric_terminator {
    () => {
        ']' | '}' | ','
    };
}

macro_rules! match_escape {
    () => {
        '\\'
    };
}

macro_rules! match_escape_non_unicode_suffix {
    () => {
        'n' | 't' | 'r' | '\\' | '/' | 'b' | 'f' | '\"'
    };
}

macro_rules! match_escape_unicode_suffix {
    () => {
        'u'
    };
}

macro_rules! match_quote {
    () => {
        '\"'
    };
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
                ch => lexer_error!(Details::InvalidCharacter(ch), self.coords),
            },
            Err(err) => match err.details {
                Details::EndOfInput => {
                    packed_token!(Token::EndOfInput, self.coords)
                }
                _ => lexer_error!(err.details, err.coords),
            },
        }
    }

    /// Match on a valid Json string.
    fn match_string(&mut self) -> ParserResult<PackedToken> {
        let start_coords = self.coords;
        loop {
            match self.advance(false) {
                Ok(_) => match self.buffer.last().unwrap() {
                    match_escape!() => match self.advance(false) {
                        Ok(_) => match self.buffer.last().unwrap() {
                            match_escape_non_unicode_suffix!() => (),
                            match_escape_unicode_suffix!() => self.check_unicode_sequence()?,
                            _ => {
                                return lexer_error!(
                                    Details::InvalidEscapeSequence(self.buffer_to_string()),
                                    self.coords
                                );
                            }
                        },
                        Err(err) => {
                            return lexer_error!(err.details, err.coords);
                        }
                    },
                    match_quote!() => {
                        return packed_token!(
                            Token::Str(self.buffer_to_string()),
                            start_coords,
                            self.coords
                        );
                    }
                    _ => (),
                },
                Err(err) => return lexer_error!(err.details, err.coords),
            }
        }
    }

    #[inline]
    fn check_unicode_sequence(&mut self) -> ParserResult<()> {
        self.advance_n(4, false).and_then(|_| {
            for i in 1..=4 {
                if !self.buffer[self.buffer.len() - i].is_ascii_hexdigit() {
                    return lexer_error!(
                        Details::InvalidUnicodeEscapeSequence(self.buffer_to_string()),
                        self.coords
                    );
                }
            }
            Ok(())
        })
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
            Ok(integral) => {
                have_decimal = !integral;
                loop {
                    match self.advance(false) {
                        Ok(_) => match self.buffer.last().unwrap() {
                            match_digit!() => (),
                            match_exponent!() => {
                                if !have_exponent {
                                    self.check_following_exponent()?;
                                    have_exponent = true;
                                } else {
                                    return lexer_error!(
                                        Details::InvalidNumericRepresentation(
                                            self.buffer_to_string()
                                        ),
                                        self.coords
                                    );
                                }
                            }
                            match_period!() => {
                                if !have_decimal {
                                    have_decimal = true;
                                } else {
                                    return lexer_error!(
                                        Details::InvalidNumericRepresentation(
                                            self.buffer_to_string()
                                        ),
                                        self.coords
                                    );
                                }
                            }
                            match_numeric_terminator!() => {
                                self.pushback();
                                break;
                            }
                            ch if ch.is_ascii_whitespace() => {
                                self.pushback();
                                break;
                            }
                            ch if ch.is_alphabetic() => {
                                return lexer_error!(
                                    Details::InvalidNumericRepresentation(self.buffer_to_string()),
                                    self.coords
                                );
                            }
                            _ => {
                                return lexer_error!(
                                    Details::InvalidNumericRepresentation(self.buffer_to_string()),
                                    self.coords
                                );
                            }
                        },
                        Err(err) => return lexer_error!(err.details, err.coords),
                    }
                }
            }
            Err(err) => {
                return lexer_error!(err.details, err.coords);
            }
        }

        self.parse_numeric(!have_decimal, start_coords, self.coords)
    }

    fn check_following_exponent(&mut self) -> ParserResult<()> {
        self.advance(false).and_then(|_| {
            return match self.buffer.last().unwrap() {
                match_plus_minus!() => Ok(()),
                _ => lexer_error!(
                    Details::InvalidNumericRepresentation(self.buffer_to_string()),
                    self.coords
                ),
            };
        })
    }

    /// Convert the contents of the buffer into an owned [String]
    #[inline]
    fn buffer_to_string(&self) -> String {
        let mut s = String::with_capacity(self.buffer.len());
        self.buffer.iter().for_each(|ch| s.push(*ch));
        s
    }

    #[inline]
    fn buffer_to_bytes_unchecked(&self) -> Vec<u8> {
        self.buffer.iter().map(|ch| *ch as u8).collect()
    }

    #[cfg(not(feature = "mixed_numerics"))]
    #[inline]
    fn parse_numeric(
        &mut self,
        integral: bool,
        start_coords: Coords,
        end_coords: Coords,
    ) -> ParserResult<PackedToken> {
        packed_token!(
            Token::Float(fast_float::parse(self.buffer_to_bytes_unchecked()).unwrap()),
            start_coords,
            end_coords
        )
    }

    #[cfg(feature = "mixed_numerics")]
    #[inline]
    fn parse_numeric(
        &mut self,
        integral: bool,
        start_coords: Coords,
        end_coords: Coords,
    ) -> ParserResult<PackedToken> {
        if integral {
            packed_token!(
                Token::Integer(lexical::parse(self.buffer_to_bytes_unchecked()).unwrap()),
                start_coords,
                end_coords
            )
        } else {
            packed_token!(
                Token::Float(fast_float::parse(self.buffer_to_bytes_unchecked()).unwrap()),
                start_coords,
                end_coords
            )
        }
    }

    /// Check that a numeric representation is prefixed correctly.
    ///
    /// A few rules here:
    /// - A leading minus must be followed by a digit
    /// - A leading minus must be followed by at most one zero before a period
    /// - Any number > zero can't have a leading zero in the representation
    #[inline]
    fn match_valid_number_prefix(&mut self) -> ParserResult<bool> {
        assert!(self.buffer[0].is_ascii_digit() || self.buffer[0] == '-');
        match self.buffer[0] {
            match_minus!() => self
                .advance(false)
                .and_then(|_| self.check_following_minus()),
            match_zero!() => self
                .advance(false)
                .and_then(|_| self.check_following_zero()),
            _ => Ok(true),
        }
    }

    #[inline]
    fn check_following_zero(&mut self) -> ParserResult<bool> {
        match self.buffer[1] {
            match_period!() => Ok(false),
            match_digit!() => lexer_error!(
                Details::InvalidNumericRepresentation(self.buffer_to_string()),
                self.coords
            ),
            _ => {
                self.pushback();
                Ok(true)
            }
        }
    }

    #[inline]
    fn check_following_minus(&mut self) -> ParserResult<bool> {
        match self.buffer[1] {
            match_non_zero_digit!() => Ok(true),
            match_zero!() => self.advance(false).and_then(|_| {
                if self.buffer[2] != '.' {
                    return lexer_error!(
                        Details::InvalidNumericRepresentation(self.buffer_to_string()),
                        self.coords
                    );
                }
                Ok(false)
            }),
            _ => lexer_error!(
                Details::InvalidNumericRepresentation(self.buffer_to_string()),
                self.coords
            ),
        }
    }

    /// Match on a null token
    fn match_null(&mut self) -> ParserResult<PackedToken> {
        let start_coords = self.coords;
        self.advance_n(3, false).and_then(|_| {
            if self.buffer[0..=3] == NULL_PATTERN {
                packed_token!(Token::Null, start_coords, self.coords)
            } else {
                lexer_error!(
                    Details::MatchFailed(
                        String::from_iter(NULL_PATTERN.iter()),
                        self.buffer_to_string()
                    ),
                    start_coords
                )
            }
        })
    }

    /// Match on a true token
    fn match_true(&mut self) -> ParserResult<PackedToken> {
        let start_coords = self.coords;
        self.advance_n(3, false).and_then(|_| {
            if self.buffer[0..=3] == TRUE_PATTERN {
                packed_token!(Token::Bool(true), start_coords, self.coords)
            } else {
                lexer_error!(
                    Details::MatchFailed(
                        String::from_iter(TRUE_PATTERN.iter()),
                        self.buffer_to_string()
                    ),
                    start_coords
                )
            }
        })
    }

    /// Match on a false token
    fn match_false(&mut self) -> ParserResult<PackedToken> {
        let start_coords = self.coords;
        self.advance_n(4, false).and_then(|_| {
            if self.buffer[0..=4] == FALSE_PATTERN {
                packed_token!(Token::Bool(false), start_coords, self.coords)
            } else {
                lexer_error!(
                    Details::MatchFailed(
                        String::from_iter(FALSE_PATTERN.iter()),
                        self.buffer_to_string()
                    ),
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
            self.advance(skip_whitespace)?;
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
                        if !c.is_ascii_whitespace() {
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
                            lexer_error!(Details::StreamFailure, self.coords)
                        }
                        DecoderErrorCode::InvalidByteSequence => {
                            lexer_error!(Details::NonUtf8InputDetected, self.coords)
                        }
                        DecoderErrorCode::EndOfInput => {
                            lexer_error!(Details::EndOfInput, self.coords)
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
    use crate::errors::{Error, ParserResult};
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
                println!("Parsing {}", l);
                let reader = reader_from_bytes!(l);
                let mut lexer = Lexer::new(reader);
                let token = lexer.consume().unwrap();
                match token.0 {
                    Token::Integer(_) => {
                        assert_eq!(
                            token.0,
                            Token::Integer(l.replace(',', "").parse::<i64>().unwrap())
                        );
                    }
                    Token::Float(_) => {
                        assert_eq!(
                            token.0,
                            Token::Float(fast_float::parse(l.replace(',', "")).unwrap())
                        );
                    }
                    _ => panic!(),
                }
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
                let mut error_token: Option<Error> = None;
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
                            println!("Dodgy string found: {} : {}", l, err.coords);
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
