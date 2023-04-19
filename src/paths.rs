use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt::Display;

/// The default separator to use within generated paths
const PATH_SEPARATOR: char = '.';

/// An enumeration fo various different path components
#[derive(PartialEq)]
pub enum JsonPathComponent<'a> {
    /// The root element '$'
    Root,
    /// A name selector component
    NameSelector(Cow<'a, str>),
    /// Wildcard selector
    WildcardSelector,
    /// Index selector
    IndexSelector(usize),
    /// Range selector
    RangeSelector(usize, usize),
}

impl<'a> Display for JsonPathComponent<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Root => write!(f, "$"),
            Self::NameSelector(s) => write!(f, "{}", s),
            Self::WildcardSelector => write!(f, "[*]"),
            Self::IndexSelector(i) => write!(f, "[{}]", i),
            Self::RangeSelector(i, j) => write!(f, "[{}..{}]", i, j),
        }
    }
}

/// Struct for creating and manipulating Json paths vaguely compatible with a subset of RFC 8259.
/// Each instance of [JsonPath] comprises of multiple [JsonPathComponent]s
pub struct JsonPath<'a> {
    /// The path components
    components: Vec<JsonPathComponent<'a>>,
}

impl<'a> JsonPath<'a> {
    /// Create a new, empty [JsonPath] instance
    pub fn new() -> Self {
        JsonPath { components: vec![] }
    }

    /// Create a new [JsonPath] instance with just a root component
    pub fn new_with_root() -> Self {
        JsonPath {
            components: vec![JsonPathComponent::Root],
        }
    }

    /// Is this an empty path?
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// A partial path doesn't have a [JsonPathComponent::Root] in the first position
    pub fn is_partial(&self) -> bool {
        if !self.is_empty() {
            self.components[0] != JsonPathComponent::Root
        } else {
            true
        }
    }

    /// Render a string ([Cow]) representation of the path
    pub fn as_string(&self) -> Cow<'a, str> {
        Cow::Owned(
            self.components
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("."),
        )
    }
}

impl<'a> Display for JsonPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

#[cfg(test)]
mod tests {

    use crate::paths::{JsonPath, JsonPathComponent};

    #[test]
    fn an_empty_path_should_be_partial() {
        let path = JsonPath::new();
        assert!(path.is_partial())
    }

    #[test]
    fn an_empty_path_should_have_an_empty_representation() {
        let path = JsonPath::new();
        assert_eq!("".to_string(), path.as_string())
    }

    #[test]
    fn a_rooted_path_should_not_be_partial() {
        let path = JsonPath::new_with_root();
        assert!(!path.is_partial())
    }

    #[test]
    fn a_rooted_path_should_have_the_correct_representation() {
        let path = JsonPath::new_with_root();
        assert_eq!("$".to_string(), path.as_string())
    }
}
