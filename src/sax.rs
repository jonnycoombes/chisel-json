use crate::coords::Coords;
use crate::decoders::DecoderSelector;
use crate::errors::{ParserError, ParserErrorDetails, ParserErrorSource, ParserResult};
use crate::events::{Event, Match};
use crate::lexer::{Lexer, Token};
use crate::paths::JsonPath;
use crate::sax_parser_error;
use crate::JsonValue;
use crate::Span;
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

macro_rules! emit_event {
    ($cb : expr, $m : expr, $span : expr, $path : expr) => {
        $cb(&Event {
            matched: $m,
            span: $span,
            path: Some(&$path),
        })
    };
    ($cb : expr, $m : expr, $span : expr) => {
        $cb(&Event {
            matched: $m,
            span: $span,
            path: None,
        })
    };
}

/// Main JSON parser struct
#[derive(Default)]
pub struct Parser {
    decoders: DecoderSelector,
}

impl Parser {
    pub fn parse_file<PathLike: AsRef<Path>, Callback>(
        &self,
        path: PathLike,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        match File::open(&path) {
            Ok(f) => {
                let mut reader = BufReader::new(f);
                let mut chars = self.decoders.default_decoder(&mut reader);
                self.parse(&mut chars, cb)
            }
            Err(_) => {
                sax_parser_error!(ParserErrorDetails::InvalidFile)
            }
        }
    }

    pub fn parse_bytes<Callback>(&self, bytes: &[u8], cb: &mut Callback) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        if bytes.is_empty() {
            return sax_parser_error!(ParserErrorDetails::ZeroLengthInput, Coords::default());
        }
        let mut reader = BufReader::new(bytes);
        let mut chars = self.decoders.default_decoder(&mut reader);
        self.parse(&mut chars, cb)
    }

    pub fn parse_str<Callback>(&self, str: &str, cb: &mut Callback) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        if str.is_empty() {
            return sax_parser_error!(ParserErrorDetails::ZeroLengthInput, Coords::default());
        }
        let mut reader = BufReader::new(str.as_bytes());
        let mut chars = self.decoders.default_decoder(&mut reader);
        self.parse(&mut chars, cb)
    }

    pub fn parse<Callback>(
        &self,
        chars: &mut impl Iterator<Item = char>,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        let mut path = JsonPath::new();
        let mut lexer = Lexer::new(chars);
        match lexer.consume()? {
            (Token::StartObject, span) => {
                emit_event!(cb, Match::StartOfInput, span)?;
                emit_event!(cb, Match::StartObject, span, path)?;
                self.parse_object(&mut lexer, &mut path, cb)
            }
            (Token::StartArray, span) => {
                emit_event!(cb, Match::StartOfInput, span, path)?;
                emit_event!(cb, Match::StartArray, span, path)?;
                self.parse_array(&mut lexer, &mut path, cb)
            }
            (_, span) => {
                sax_parser_error!(ParserErrorDetails::InvalidRootObject, span.start)
            }
        }
    }

    fn parse_value<Callback>(
        &self,
        lexer: &mut Lexer,
        path: &mut JsonPath,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        match lexer.consume()? {
            (Token::StartObject, span) => {
                emit_event!(cb, Match::StartObject, span, path)?;
                self.parse_object(lexer, path, cb)
            }
            (Token::StartArray, span) => {
                emit_event!(cb, Match::StartArray, span, path)?;
                self.parse_array(lexer, path, cb)
            }
            (Token::Str(str), span) => {
                emit_event!(cb, Match::String(Cow::Borrowed(&str)), span, path)
            }
            (Token::Float(value), span) => {
                emit_event!(cb, Match::Float(value), span, path)
            }
            (Token::Integer(value), span) => {
                emit_event!(cb, Match::Integer(value), span, path)
            }
            (Token::Boolean(value), span) => {
                emit_event!(cb, Match::Boolean(value), span, path)
            }
            (Token::Null, span) => {
                emit_event!(cb, Match::Null, span, path)
            }
            (token, span) => {
                sax_parser_error!(ParserErrorDetails::UnexpectedToken(token), span.start)
            }
        }
    }

    /// An object is just a list of comma separated KV pairs
    fn parse_object<Callback>(
        &self,
        lexer: &mut Lexer,
        path: &mut JsonPath,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        loop {
            match lexer.consume()? {
                (Token::Str(str), span) => {
                    path.push_str_selector(&str);
                    emit_event!(cb, Match::ObjectKey(Cow::Borrowed(&str)), span, path)?;
                    let should_be_colon = lexer.consume()?;
                    match should_be_colon {
                        (Token::Colon, _) => {
                            self.parse_value(lexer, path, cb)?;
                            path.pop();
                        }
                        (_, _) => {
                            return sax_parser_error!(
                                ParserErrorDetails::PairExpected,
                                should_be_colon.1.start
                            )
                        }
                    }
                }
                (Token::Comma, _) => (),
                (Token::EndObject, span) => {
                    return emit_event!(cb, Match::EndObject, span, path);
                }
                (_token, span) => {
                    return sax_parser_error!(ParserErrorDetails::InvalidArray, span.start)
                }
            }
        }
    }

    /// An array is just a list of comma separated values
    fn parse_array<Callback>(
        &self,
        lexer: &mut Lexer,
        path: &mut JsonPath,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        let mut index = 0;
        loop {
            path.push_index_select(index);
            match lexer.consume()? {
                (Token::StartArray, span) => {
                    emit_event!(cb, Match::StartArray, span, path)?;
                    self.parse_array(lexer, path, cb)?;
                }
                (Token::EndArray, span) => {
                    path.pop();
                    return emit_event!(cb, Match::EndArray, span, path);
                }
                (Token::StartObject, span) => {
                    emit_event!(cb, Match::StartObject, span, path)?;
                    self.parse_object(lexer, path, cb)?;
                }
                (Token::Str(str), span) => {
                    emit_event!(cb, Match::String(Cow::Borrowed(&str)), span, path)?;
                }
                (Token::Float(value), span) => {
                    emit_event!(cb, Match::Float(value), span, path)?;
                }
                (Token::Integer(value), span) => {
                    emit_event!(cb, Match::Integer(value), span, path)?;
                }
                (Token::Boolean(value), span) => {
                    emit_event!(cb, Match::Boolean(value), span, path)?;
                }
                (Token::Null, span) => emit_event!(cb, Match::Null, span, path)?,
                (Token::Comma, _) => index += 1,
                (_token, span) => {
                    return sax_parser_error!(ParserErrorDetails::InvalidArray, span.start);
                }
            }
            path.pop();
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::decoders::DecoderSelector;
    use crate::errors::ParserErrorDetails;
    use crate::relative_file;
    use crate::sax::Parser;
    use bytesize::ByteSize;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;
    use std::time::Instant;
    use std::{env, fs};

    #[test]
    fn should_puke_on_empty_input() {
        let input = "";
        let parser = Parser::default();
        let parsed = parser.parse_str(input, &mut |_e| Ok(()));
        assert!(parsed.is_err());
        assert_eq!(
            parsed.err().unwrap().details,
            ParserErrorDetails::ZeroLengthInput
        );
    }

    #[test]
    fn should_parse_successfully() {
        let mut counter = 0;
        let path = relative_file!("fixtures/json/valid/events.json");
        let parser = Parser::default();
        let parsed = parser.parse_file(&path, &mut |_e| {
            counter += 1;
            Ok(())
        });
        println!("{} SAX events processed", counter);
        assert!(parsed.is_ok());
    }

    #[test]
    fn should_successfully_bail() {
        let path = relative_file!("fixtures/json/invalid/invalid_1.json");
        let parser = Parser::default();
        let parsed = parser.parse_file(&path, &mut |_e| Ok(()));
        println!("Parse result = {:?}", parsed);
        assert!(parsed.is_err());
        assert!(parsed.err().unwrap().details == ParserErrorDetails::InvalidRootObject);
    }
}
