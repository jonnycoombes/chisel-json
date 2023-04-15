use crate::coords::Span;
use crate::errors::Error;
use std::borrow::Cow;

/// Enumeration of the various different matches that can be produced during a parse
pub enum Match<'a> {
    StartOfInput,
    EndOfInput,
    StartObject,
    ObjectKey(Cow<'a, str>),
    EndObject,
    StartArray,
    ArrayString(usize, Cow<'a, str>),
    ArrayNum(usize, f64),
    ArrayBool(usize, bool),
    ArrayNull(usize),
    EndArray,
    String(Cow<'a, str>),
    Num(f64),
    Bool(bool),
    Null,
}

/// A general event produced by the parser during a parse
pub struct Event<'a> {
    /// The [Match] associated with the event
    pub matched: Match<'a>,

    /// The [Span] associated with the [matched]
    pub span: Span,
}

/// Trait that should be implemented by anything sinking events from the parser
pub trait EventSink {
    fn on_parse_event(event: &Event);

    fn on_parse_error(error: &Error);
}
