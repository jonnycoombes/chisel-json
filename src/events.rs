use crate::coords::Span;
use crate::errors::ParserError;
use crate::paths::JsonPath;
use std::borrow::Cow;
use std::fmt::Display;

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
    Boolean(bool),
    /// Emitted when a null is matched
    Null,
}

impl<'a> Display for Match<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Match::StartOfInput => write!(f, "StartOfInput"),
            Match::EndOfInput => write!(f, "EndOfInput"),
            Match::StartObject => write!(f, "StartObject"),
            Match::ObjectKey(_) => write!(f, "ObjectKey"),
            Match::EndObject => write!(f, "EndObject"),
            Match::StartArray => write!(f, "StartArray"),
            Match::EndArray => write!(f, "EndArray"),
            Match::String(value) => write!(f, "String({})", value),
            Match::Integer(value) => write!(f, "Integer({})", value),
            Match::Float(value) => write!(f, "Float({})", value),
            Match::Boolean(b) => write!(f, "Boolean({})", b),
            Match::Null => write!(f, "Null"),
        }
    }
}

/// A general event produced by the parser during a parse
pub struct Event<'a> {
    /// The [Match] associated with the event
    pub matched: Match<'a>,

    /// The [Span] associated with the [matched]
    pub span: Span,

    /// Optional [JsonPath] information relating to the event
    pub path: Option<&'a JsonPath<'a>>,
}

impl<'a> Event<'a> {
    /// Checks whether an event has a path or not
    fn has_path(&self) -> bool {
        self.path.is_some()
    }
}

impl<'a> Display for Event<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_some() {
            write!(
                f,
                "Event[{}, {}, {}]",
                self.matched,
                self.span,
                self.path.unwrap()
            )
        } else {
            write!(f, "Event[{}, {}]", self.matched, self.span)
        }
    }
}
