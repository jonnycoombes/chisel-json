#![allow(unused_macros)]

use chisel_stringtable::btree_string_table::BTreeStringTable;
use std::borrow::Cow;
use std::cell::{RefCell, RefMut};
use std::fmt::Debug;
use std::io::{BufRead, Read};
use std::rc::Rc;
use std::sync::Arc;

use chisel_stringtable::common::StringTable;

use crate::coords::{Coords, Span};
use crate::errors::ParserResult;
use crate::errors::*;
use crate::scanner::{Lexeme, PackedLexeme, Scanner, ScannerMode};
use crate::{is_digit, is_non_alphabetic, is_period, is_whitespace, lexer_error, unpack_digit};

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

/// Default string buffer capacity
const DEFAULT_BUFFER_CAPACITY: usize = 1024;

/// A lexer implementation which will consume a stream of lexemes from a [Scanner] and produce
/// a stream of [Token]s.
#[derive()]
pub struct LexerOld<B: BufRead> {
    /// [StringTable] used for interning all parsed strings
    strings: Rc<RefCell<dyn StringTable<'static, u64>>>,
    /// The [Scanner] instance used by the lexer to source [Lexeme]s
    scanner: Scanner<B>,
    /// Internal buffer for hoovering up strings from the input
    buffer: RefCell<String>,
}

impl<B: BufRead> LexerOld<B> {
    /// Construct a new *validating* [LexerOld] instance which will utilise a given [StringTable]
    pub fn new(reader: B) -> Self {
        LexerOld {
            strings: Rc::new(RefCell::new(BTreeStringTable::new())),
            scanner: Scanner::new(reader),
            buffer: RefCell::new(String::with_capacity(DEFAULT_BUFFER_CAPACITY)),
        }
    }

    /// Get a smart pointer to the internal string table used to intern strings used by the lexer
    pub fn lookup_string(&self, key: u64) -> Cow<'static, str> {
        self.strings.borrow().get(key).unwrap().clone()
    }

    /// Consume the next token from the input stream. This is a simple LA(1) algorithm,
    /// which looks ahead in the input 1 lexeme, and then based on the grammar rules, attempts
    /// to consume a token based on the prefix found. The working assumption is that ws is skipped
    /// unless parsing out specific types of tokens such as strings, numbers etc...
    pub fn consume(&self) -> ParserResult<PackedToken> {
        match self
            .scanner
            .with_mode(ScannerMode::IgnoreWhitespace)
            .lookahead(1)
        {
            Ok(packed) => match packed.0 {
                Lexeme::LeftBrace => self.match_start_object(),
                Lexeme::RightBrace => self.match_end_object(),
                Lexeme::LeftBracket => self.match_start_array(),
                Lexeme::RightBracket => self.match_end_array(),
                Lexeme::DoubleQuote => self.match_string(),
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
    fn match_exact(&self, seq: &[Lexeme]) -> ParserResult<(Coords, Coords)> {
        for (index, c) in seq.iter().enumerate() {
            match self.scanner.lookahead(index + 1) {
                Ok(packed) => {
                    if packed.0 != *c {
                        return lexer_error!(
                            ParserErrorCode::InvalidLexeme,
                            format!("was looking for {}, found {}", c, packed.0),
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
                if seq.contains(&packed.0) {
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
                if is_digit!(la.0) {
                    Ok(())
                } else {
                    lexer_error!(
                        ParserErrorCode::MatchFailed,
                        "invalid numeric prefix found",
                        la.1
                    )
                }
            }
            '0' => {
                if !is_digit!(la.0) && !is_period!(la.0) {
                    return Ok(());
                }
                if !is_period!(la.0) {
                    lexer_error!(
                        ParserErrorCode::MatchFailed,
                        "invalid numeric prefix found",
                        la.1
                    )
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    /// Attempt to match on a number representation.  Utilise the excellent fast_float lib in order
    /// to carry out the actual parsing of the numeric value
    fn match_number(&self, first: char) -> ParserResult<PackedToken> {
        let mut buffer = self.buffer.borrow_mut();
        buffer.clear();
        buffer.push(first);
        let start_coords = self
            .scanner
            .with_mode(ScannerMode::ProduceWhitespace)
            .consume()?
            .1;
        let mut lookahead = 1;
        self.match_valid_number_prefix(first)?;
        loop {
            lookahead += 1;
            let packed = self.scanner.lookahead(lookahead)?;
            match packed.0 {
                Lexeme::Period => buffer.push('.'),
                Lexeme::Digit(d) => buffer.push(d),
                Lexeme::Minus => buffer.push('-'),
                Lexeme::Plus => buffer.push('+'),
                Lexeme::Alphabetic(c) => match c {
                    'e' | 'E' => buffer.push(c),
                    _ => {
                        return lexer_error!(
                            ParserErrorCode::EndOfInput,
                            "invalid character found whilst parsing number",
                            packed.1
                        );
                    }
                },
                Lexeme::NewLine | Lexeme::Comma => break,
                Lexeme::Whitespace(_) => break,
                Lexeme::LeftBrace | Lexeme::RightBrace => break,
                Lexeme::LeftBracket | Lexeme::RightBracket => break,
                Lexeme::EndOfInput => {
                    return lexer_error!(
                        ParserErrorCode::EndOfInput,
                        "end of input found whilst parsing string",
                        packed.1
                    );
                }
                _ => break,
            }
        }

        self.scanner.discard(lookahead);
        match fast_float::parse(buffer.as_bytes()) {
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
    /// original format. This function does perform validation on the contents of the string, to
    /// ensure that any escape sequences etc...are well-formed.  This path is should be slower than
    /// the raw string matching function
    fn match_string(&self) -> ParserResult<PackedToken> {
        let mut buffer = self.buffer.borrow_mut();
        buffer.clear();
        buffer.push('\"');

        let start_coords = self
            .scanner
            .with_mode(ScannerMode::ProduceWhitespace)
            .consume()?
            .1;

        loop {
            let packed = self.scanner.consume()?;
            match packed.0 {
                Lexeme::Escape => self.match_escape_sequence(&mut buffer)?,
                Lexeme::DoubleQuote => {
                    buffer.push('\"');
                    break;
                }
                Lexeme::NonAlphabetic(c) => buffer.push(c),
                Lexeme::Alphabetic(c) => buffer.push(c),
                Lexeme::Digit(c) => buffer.push(c),
                Lexeme::Whitespace(c) => buffer.push(c),
                Lexeme::LeftBrace => buffer.push('{'),
                Lexeme::RightBrace => buffer.push('}'),
                Lexeme::LeftBracket => buffer.push('['),
                Lexeme::RightBracket => buffer.push(']'),
                Lexeme::Plus => buffer.push('+'),
                Lexeme::Minus => buffer.push('-'),
                Lexeme::Colon => buffer.push(':'),
                Lexeme::Comma => buffer.push(':'),
                Lexeme::Period => buffer.push('.'),
                Lexeme::SingleQuote => buffer.push('\''),
                Lexeme::EndOfInput => {
                    return lexer_error!(
                        ParserErrorCode::EndOfInput,
                        "end of input found whilst parsing string",
                        packed.1
                    );
                }
                Lexeme::NewLine => {
                    return lexer_error!(
                        ParserErrorCode::EndOfInput,
                        "newline found whilst parsing string",
                        packed.1
                    );
                }
                _ => break,
            }
        }

        Ok(packed_token!(
            Token::Str(self.compute_intern_hash(buffer.as_str())),
            start_coords,
            self.scanner.back_coords()
        ))
    }

    /// Compute a hash value for a given string slice, either a pre-existing interned string,
    /// or a new addition to the string table
    #[inline(always)]
    fn compute_intern_hash(&self, value: &str) -> u64 {
        let mut strings = self.strings.borrow_mut();
        match strings.contains(value) {
            Some(h) => h,
            None => strings.add(value),
        }
    }

    /// Match a valid string escape sequence
    fn match_escape_sequence(&self, buffer: &mut RefMut<String>) -> ParserResult<()> {
        buffer.push('\\');
        let packed = self.scanner.consume()?;
        match packed.0 {
            Lexeme::DoubleQuote => buffer.push('\"'),
            Lexeme::Alphabetic(c) => match c {
                'u' => {
                    buffer.push(c);
                    self.match_unicode_escape_sequence(buffer)?
                }
                'n' | 't' | 'r' | '\\' | '/' | 'b' | 'f' => buffer.push(c),
                _ => {
                    return lexer_error!(
                        ParserErrorCode::InvalidCharacter,
                        "invalid escape sequence detected",
                        packed.1
                    );
                }
            },
            _ => (),
        }
        Ok(())
    }

    /// Match a valid unicode escape sequence in the form uXXXX where each X is a valid hex
    /// digit
    fn match_unicode_escape_sequence(&self, buffer: &mut RefMut<String>) -> ParserResult<()> {
        for _ in 1..=4 {
            let packed = self.scanner.consume()?;
            match packed.0 {
                Lexeme::Alphabetic(c) | Lexeme::Digit(c) => {
                    if c.is_ascii_hexdigit() {
                        buffer.push(c);
                    } else {
                        return lexer_error!(
                            ParserErrorCode::InvalidCharacter,
                            "invalid hex escape code detected",
                            packed.1
                        );
                    }
                }
                _ => {
                    return lexer_error!(
                        ParserErrorCode::InvalidCharacter,
                        "invalid escape sequence detected",
                        packed.1
                    );
                }
            }
        }
        Ok(())
    }

    /// Consume a null token from the input and and return a [PackedToken]
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
        Ok(packed_token!(Token::StartObject, result.1))
    }

    /// Consume a right brace from the input and and return a [PackedToken]
    fn match_end_object(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::EndObject, result.1))
    }

    /// Consume a left bracket from the input and and return a [PackedToken]
    fn match_start_array(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::StartArray, result.1))
    }

    /// Consume a right bracket from the input and and return a [PackedToken]
    fn match_end_array(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::EndArray, result.1))
    }

    /// Consume a comma from the input and and return a [PackedToken]
    fn match_comma(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::Comma, result.1))
    }

    /// Consume a colon from the input and and return a [PackedToken]
    fn match_colon(&self) -> ParserResult<PackedToken> {
        let result = self.scanner.consume()?;
        Ok(packed_token!(Token::Colon, result.1))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::env;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::rc::Rc;

    use chisel_stringtable::btree_string_table::BTreeStringTable;
    use chisel_stringtable::common::StringTable;

    use crate::coords::{Coords, Span};
    use crate::errors::{ParserError, ParserResult};
    use crate::lexer_old::{Lexer, PackedToken, Token};
    use crate::{lines_from_relative_file, reader_from_bytes};

    #[test]
    fn should_parse_basic_tokens() {
        let reader = reader_from_bytes!("{}[],:");
        let lexer = Lexer::new(reader);
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
        let lexer = Lexer::new(reader);
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
                let lexer = Lexer::new(reader);
                let token = lexer.consume().unwrap();
                match token.0 {
                    Token::Str(hash) => {
                        assert_eq!(lexer.lookup_string(hash), l.as_str())
                    }
                    _ => panic!(),
                }
            }
        }
    }

    #[test]
    fn should_parse_numerics() {
        let lines = lines_from_relative_file!("fixtures/utf-8/numbers.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                let reader = reader_from_bytes!(l);
                let lexer = Lexer::new(reader);
                let token = lexer.consume().unwrap();
                assert_eq!(
                    token.0,
                    Token::Num(fast_float::parse(l.replace(',', "")).unwrap())
                );
            }
        }
    }

    #[test]
    fn should_correctly_handle_invalid_numbers() {
        let lines = lines_from_relative_file!("fixtures/utf-8/invalid_numbers.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                let reader = reader_from_bytes!(l);
                let lexer = Lexer::new(reader);
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
                let lexer = Lexer::new(reader);
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
        let lexer = Lexer::new(reader);
        let mut results: Vec<ParserResult<PackedToken>> = vec![];
        for _ in 1..=2 {
            results.push(lexer.consume());
        }
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        println!("Parse error: {:?}", results[1]);
    }
}
