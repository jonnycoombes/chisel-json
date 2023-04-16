use crate::coords::Span;
use crate::errors::Error;
use std::borrow::Cow;

/// Enumeration of the various different matches that can be produced during a parse
pub enum Match<'a> {
    /// Start of the input Emitted prior to anything else
    StartOfInput,
    /// End of the input Emitted after everything else
    EndOfInput,
    /// Emitted when the start of a new object is matched
    StartObject,
    /// Emitted when a new key within an object is matched
    ObjectKey(Cow<'a, str>),
    /// Emitted after an object has been fully parsed
    EndObject,
    /// Emitted when the start of an array is matched
    StartArray,
    /// Emitted when the end of an array is matched
    EndArray,
    /// Emitted when a string is matched
    String(Cow<'a, str>),
    /// Emitted when an integer is matched
    Integer(i64),
    /// Emitted when a float is matched
    Float(f64),
    /// Emitted when a boolean is matched
    Bool(bool),
    /// Emitted when a null is matched
    Null,
}

/// A general event produced by the parser during a parse
pub struct Event<'a> {
    /// The [Match] associated with the event
    pub matched: Match<'a>,

    /// The [Span] associated with the [matched]
    pub span: Span,
}
