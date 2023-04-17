use crate::errors::{Details, Error, ParserResult, Stage};
use crate::events::{Event, Match};
use crate::lexer::{Lexer, Token};
use crate::parser_error;
use crate::paths::PathElementStack;
use crate::JsonValue;
use crate::Span;
use std::borrow::Cow;
use std::io::BufRead;

macro_rules! emit_event {
    ($cb : expr, $m : expr, $span : expr) => {
        $cb(&Event {
            matched: $m,
            span: $span,
        })
    };
}

/// Main JSON parser struct
#[derive(Default)]
pub struct Parser {
    /// A stack for tracking the current path within the parsed JSON
    path: PathElementStack,
}

impl Parser {
    pub fn parse<Buffer: BufRead, Callback, OnError>(
        &self,
        input: Buffer,
        cb: &mut Callback,
        err_cb: &mut OnError,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
        OnError: FnMut(&Error),
    {
        let mut lexer = Lexer::new(input);
        self.parse_value(&mut lexer, cb, err_cb)
    }

    fn parse_value<Buffer: BufRead, Callback, OnError>(
        &self,
        lexer: &mut Lexer<Buffer>,
        cb: &mut Callback,
        err_cb: &mut OnError,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
        OnError: FnMut(&Error),
    {
        match lexer.consume()? {
            (Token::StartObject, span) => {
                emit_event!(cb, Match::StartObject, span)?;
                self.parse_object(lexer, cb, err_cb)
            }
            (Token::StartArray, span) => {
                emit_event!(cb, Match::StartArray, span)?;
                self.parse_array(lexer, cb, err_cb)
            }
            (Token::Str(str), span) => emit_event!(cb, Match::String(Cow::Borrowed(&str)), span),
            (Token::Float(value), span) => emit_event!(cb, Match::Float(value), span),
            (Token::Integer(value), span) => emit_event!(cb, Match::Integer(value), span),
            (Token::Bool(value), span) => emit_event!(cb, Match::Bool(value), span),
            (Token::Null, span) => emit_event!(cb, Match::Null, span),
            (token, span) => {
                parser_error!(Details::UnexpectedToken(token), span.start)
            }
        }
    }

    /// An object is just a list of comma separated KV pairs
    fn parse_object<Buffer: BufRead, Callback, OnError>(
        &self,
        lexer: &mut Lexer<Buffer>,
        cb: &mut Callback,
        err_cb: &mut OnError,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
        OnError: FnMut(&Error),
    {
        loop {
            match lexer.consume()? {
                (Token::Str(str), span) => {
                    emit_event!(cb, Match::ObjectKey(Cow::Borrowed(&str)), span)?;
                    let should_be_colon = lexer.consume()?;
                    match should_be_colon {
                        (Token::Colon, _) => self.parse_value(lexer, cb, err_cb)?,
                        (_, _) => {
                            return parser_error!(Details::PairExpected, should_be_colon.1.start)
                        }
                    }
                }
                (Token::Comma, _) => (),
                (Token::EndObject, span) => {
                    return emit_event!(cb, Match::EndObject, span);
                }
                (_token, span) => return parser_error!(Details::InvalidArray, span.start),
            }
        }
    }

    /// An array is just a list of comma separated values
    fn parse_array<Buffer: BufRead, Callback, OnError>(
        &self,
        lexer: &mut Lexer<Buffer>,
        cb: &mut Callback,
        err_cb: &mut OnError,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
        OnError: FnMut(&Error),
    {
        loop {
            match lexer.consume()? {
                (Token::StartArray, span) => {
                    emit_event!(cb, Match::StartArray, span)?;
                    self.parse_array(lexer, cb, err_cb)?
                }
                (Token::EndArray, span) => {
                    return emit_event!(cb, Match::EndArray, span);
                }
                (Token::StartObject, span) => {
                    emit_event!(cb, Match::StartObject, span)?;
                    self.parse_object(lexer, cb, err_cb)?
                }
                (Token::Str(str), span) => {
                    emit_event!(cb, Match::String(Cow::Borrowed(&str)), span)?
                }
                (Token::Float(value), span) => cb(&Event {
                    matched: Match::Float(value),
                    span,
                })?,
                (Token::Integer(value), span) => emit_event!(cb, Match::Integer(value), span)?,
                (Token::Bool(value), span) => emit_event!(cb, Match::Bool(value), span)?,
                (Token::Null, span) => emit_event!(cb, Match::Null, span)?,
                (Token::Comma, _) => (),
                (_token, span) => {
                    return parser_error!(Details::InvalidArray, span.start);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::sax::Parser;
    use crate::{reader_from_file, reader_from_relative_file};
    use bytesize::ByteSize;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;
    use std::time::Instant;
    use std::{env, fs};

    #[test]
    fn should_parse_successfully() {
        let mut counter = 0;
        let reader = reader_from_relative_file!("fixtures/json/valid/canada.json");
        let parser = Parser::default();
        let parsed = parser.parse(
            reader,
            &mut |e| {
                counter += 1;
                println!("(Event: {:?}, Counter: {})", e.matched, counter);
                Ok(())
            },
            &mut |_err| (),
        );
        assert!(parsed.is_ok());
    }
}
