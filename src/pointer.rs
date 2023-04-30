//! A representation of a JSON Pointer with associated operations, as per RFC 6901
//!
//!
use std::{borrow::Cow, collections::VecDeque, fmt::Display, ops::Add};

/// Each pointer is a series of segments delineated by a separator char
const PATH_SEPARATOR: char = '/';
/// As per the RFC, we need to encode any tilde characters as ~0
const ENCODED_TILDE: &str = "~0";
/// As per the RFC, we need to encode any slash characters as ~1
const ENCODED_SLASH: &str = "~1";

/// Each pointer is made of one of three different component types
#[derive(Clone, PartialEq)]
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
            Self::Root => write!(f, ""),
            Self::Name(s) => write!(
                f,
                "{}",
                &s.replace("/", ENCODED_SLASH).replace("~", ENCODED_TILDE)
            ),
            Self::Index(i) => write!(f, "{}", i),
        }
    }
}

/// A structure representing a complete pointer, comprising multiple [JsonPointerComponent]s
#[derive(Default)]
pub struct JsonPointer<'a> {
    /// The components that go together to make up the pointer
    components: VecDeque<JsonPointerComponent<'a>>,
}

impl<'a> JsonPointer<'a> {
    /// Returns the number of [JsonPointerComponent]s within the pointer
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Checks whether the pointer is the empty pointer
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Push a new [JsonPointerComponent::Name] onto the end of the pointer
    pub fn push_name(&mut self, name: &'a str) {
        self.components
            .push_back(JsonPointerComponent::Name(Cow::Borrowed(name)))
    }

    /// Push a new [JsonPointerComponent::Index] onto the end of the pointer
    pub fn push_index(&mut self, index: usize) {
        self.components
            .push_back(JsonPointerComponent::Index(index))
    }

    /// Checks whether a path matches another path.
    pub fn matches(&self, rhs: &'a JsonPointer) -> bool {
        self.as_str() == rhs.as_str()
    }

    /// Serialise the pointer
    pub fn as_str(&self) -> Cow<'a, str> {
        todo!()
    }
}

impl<'a> Add<&JsonPointer<'a>> for JsonPointer<'a> {
    type Output = Self;

    /// Concatenate two [JsonPointer] instances.
    fn add(mut self, rhs: &JsonPointer<'a>) -> Self {
        rhs.components
            .iter()
            .for_each(|c| self.components.push_back(c.clone()));
        self
    }
}

#[cfg(test)]
mod tests {}
