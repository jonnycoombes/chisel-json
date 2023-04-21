use crate::coords::Coords;
use crate::errors::{Details, Error, ParserResult, Stage};
use crate::events::{Event, Match};
use crate::lexer::{Lexer, Token};
use crate::parser_error;
use crate::paths::JsonPath;
use crate::JsonValue;
use crate::Span;
use std::borrow::Cow;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

macro_rules! emit_event {
    ($cb : expr, $m : expr, $span : expr, $path : expr) => {
        $cb(
            &Event {
                matched: $m,
                span: $span,
            },
            &$path,
        )
    };
}

/// Main JSON parser struct
#[derive(Default)]
pub struct Parser {}

impl Parser {
    pub fn parse_file<PathLike: AsRef<Path>, Callback>(
        &self,
        path: PathLike,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event, &JsonPath) -> ParserResult<()>,
    {
        match File::open(&path) {
            Ok(f) => {
                let reader = BufReader::new(f);
                self.parse(reader, cb)
            }
            Err(_) => {
                parser_error!(Details::InvalidFile, Coords::default())
            }
        }
    }

    pub fn parse_bytes<Callback>(&self, bytes: &[u8], cb: &mut Callback) -> ParserResult<()>
    where
        Callback: FnMut(&Event, &JsonPath) -> ParserResult<()>,
    {
        if bytes.is_empty() {
            return parser_error!(Details::ZeroLengthInput, Coords::default());
        }
        let reader = BufReader::new(bytes);
        self.parse(reader, cb)
    }

    pub fn parse_str<Callback>(&self, str: &str, cb: &mut Callback) -> ParserResult<()>
    where
        Callback: FnMut(&Event, &JsonPath) -> ParserResult<()>,
    {
        if str.is_empty() {
            return parser_error!(Details::ZeroLengthInput, Coords::default());
        }
        let reader = BufReader::new(str.as_bytes());
        self.parse(reader, cb)
    }

    fn parse<Buffer: BufRead, Callback>(&self, input: Buffer, cb: &mut Callback) -> ParserResult<()>
    where
        Callback: FnMut(&Event, &JsonPath) -> ParserResult<()>,
    {
        let mut path = JsonPath::new();
        let mut lexer = Lexer::new(input);
        match lexer.consume()? {
            (Token::StartObject, span) => {
                emit_event!(cb, Match::StartOfInput, span, path)?;
                emit_event!(cb, Match::StartObject, span, path)?;
                self.parse_object(&mut lexer, &mut path, cb)
            }
            (Token::StartArray, span) => {
                emit_event!(cb, Match::StartOfInput, span, path)?;
                emit_event!(cb, Match::StartArray, span, path)?;
                self.parse_array(&mut lexer, &mut path, cb)
            }
            (_, span) => {
                parser_error!(Details::InvalidRootObject, span.start)
            }
        }
    }

    fn parse_value<Buffer: BufRead, Callback>(
        &self,
        lexer: &mut Lexer<Buffer>,
        path: &mut JsonPath,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event, &JsonPath) -> ParserResult<()>,
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
                parser_error!(Details::UnexpectedToken(token), span.start)
            }
        }
    }

    /// An object is just a list of comma separated KV pairs
    fn parse_object<Buffer: BufRead, Callback>(
        &self,
        lexer: &mut Lexer<Buffer>,
        path: &mut JsonPath,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event, &JsonPath) -> ParserResult<()>,
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
                            return parser_error!(Details::PairExpected, should_be_colon.1.start)
                        }
                    }
                }
                (Token::Comma, _) => (),
                (Token::EndObject, span) => {
                    return emit_event!(cb, Match::EndObject, span, path);
                }
                (_token, span) => return parser_error!(Details::InvalidArray, span.start),
            }
        }
    }

    /// An array is just a list of comma separated values
    fn parse_array<Buffer: BufRead, Callback>(
        &self,
        lexer: &mut Lexer<Buffer>,
        path: &mut JsonPath,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event, &JsonPath) -> ParserResult<()>,
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
                    return parser_error!(Details::InvalidArray, span.start);
                }
            }
            path.pop();
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::errors::Details;
    use crate::sax::Parser;
    use crate::{reader_from_file, reader_from_relative_file};
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
        assert_eq!(parsed.err().unwrap().details, Details::ZeroLengthInput);
    }

    #[test]
    fn should_parse_successfully() {
        let mut counter = 0;
        let reader = reader_from_relative_file!("fixtures/json/valid/canada.json");
        let parser = Parser::default();
        let parsed = parser.parse(reader, &mut |_e| {
            counter += 1;
            Ok(())
        });
        println!("{} SAX events processed", counter);
        assert!(parsed.is_ok());
    }

    #[test]
    fn should_successfully_bail() {
        let reader = reader_from_file!("fixtures/json/invalid/invalid_1.json");
        let parser = Parser::default();
        let parsed = parser.parse(reader, &mut |e| {
            println!("SAX event = {:?}", e);
            Ok(())
        });
        println!("Parse result = {:?}", parsed);
        assert!(parsed.is_err());
        assert!(parsed.err().unwrap().details == Details::InvalidRootObject);
    }
}
