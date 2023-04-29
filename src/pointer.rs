//! A representation of a JSON Pointer with associated operations, as per RFC 6901
//!
//!
use std::{borrow::Cow, collections::VecDeque, fmt::Display};

/// Each pointer is a series of segments delineated by a separator char
const PATH_SEPARATOR: char = '/';
/// As per the RFC, we need to encode any tilde characters as ~0
const ENCODED_TILDE: &str = "~0";
/// As per the RFC, we need to encode any slash characters as ~1
const ENCODED_SLASH: &str = "~1";

/// Each pointer is made of one of three different component types
#[derive(Clone)]
pub enum JsonPointerComponent<'a> {
    /// The root element of a pointer
    Root,
    /// A named element within a pointer
    Name(Cow<'a, str>),
    /// An indexed element within a pointer
    Index(usize),
}

impl<'a> Display for JsonPointerComponent<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Root => write!(f, "/"),
            Self::Name(s) => write!(f, "{}", &s),
            Self::Index(i) => write!(f, "{}", i),
        }
    }
}

/// A structure representing a complete pointer, comprising multiple [JsonPointerComponent]s
pub struct JsonPointer<'a> {
    /// The components that go together to make up the pointer
    components: VecDeque<JsonPointerComponent<'a>>,
}

impl<'a> JsonPointer<'a> {}

#[cfg(test)]
mod tests {}
