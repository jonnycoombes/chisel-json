use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt::Display;
use std::ops::Add;

/// The default separator to use within generated paths
const PATH_SEPARATOR: char = '.';

/// An enumeration fo various different path components
#[derive(Clone, PartialEq)]
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
    /// Create a new, partial (no root) [JsonPath] instance
    pub fn new_partial() -> Self {
        JsonPath { components: vec![] }
    }

    /// Create a new [JsonPath] instance with just a root component
    pub fn new() -> Self {
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

    /// Push a new [JsonPathComponent::NameSelector] based on a string slice.  Note that no
    /// validation is carried out (as of yet) to check whether the name is actually valid and not
    /// full of crap
    pub fn push_str_selector(mut self, name: &'a str) -> Self {
        self.components
            .push(JsonPathComponent::NameSelector(Cow::from(name)));
        self
    }

    /// Push a new [JsonPathComponent::IndexSelector] based on a given index
    pub fn push_index_select(mut self, index: usize) -> Self {
        self.components
            .push(JsonPathComponent::IndexSelector(index));
        self
    }

    /// Push a new [JsonPathComponent::RangeSelector] based on a given start and end index
    pub fn push_range_selector(mut self, start: usize, end: usize) -> Self {
        self.components
            .push(JsonPathComponent::RangeSelector(start, end));
        self
    }

    /// Push a new [JsonPathComponent::WildcardSelector]
    pub fn push_wildcard_selector(mut self) -> Self {
        self.components.push(JsonPathComponent::WildcardSelector);
        self
    }

    /// Appends a new [JsonPathComponent] to the end of the path
    pub fn push(mut self, component: JsonPathComponent<'a>) -> Self {
        self.components.push(component);
        self
    }

    /// Pops the last [JsonPathComponent] from the end of the path (if it exists)
    pub fn pop(&mut self) -> Option<JsonPathComponent<'a>> {
        self.components.pop()
    }
}

impl<'a> Display for JsonPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl<'a> Add<&JsonPath<'a>> for JsonPath<'a> {
    type Output = Self;

    /// Concatenate two [JsonPath] instances.  Does some basic checking to ensure that you don't
    /// try and concatenate two rooted paths, or concatenate a rooted path to a partial path
    fn add(mut self, rhs: &JsonPath<'a>) -> Self {
        if !self.is_partial() && !rhs.is_partial() {
            panic!("attempted to concatenate two rooted paths")
        }
        if self.is_partial() && !rhs.is_partial() {
            panic!("attempted to concatenate a rooted path to a partial path")
        }
        rhs.components
            .iter()
            .for_each(|c| self.components.push(c.clone()));
        self
    }
}

#[cfg(test)]
mod tests {

    use crate::paths::{JsonPath, JsonPathComponent};

    #[test]
    fn a_new_partial_path_should_be_partial() {
        let path = JsonPath::new_partial();
        assert!(path.is_partial())
    }

    #[test]
    fn a_new_partial_path_should_have_an_empty_representation() {
        let path = JsonPath::new_partial();
        assert_eq!("".to_string(), path.as_string())
    }

    #[test]
    fn a_new_partial_path_should_be_empty() {
        let path = JsonPath::new_partial();
        assert!(path.is_empty())
    }

    #[test]
    fn a_new_rooted_path_should_not_be_partial() {
        let path = JsonPath::new();
        assert!(!path.is_partial())
    }

    #[test]
    fn a_new_rooted_path_should_have_the_correct_representation() {
        let path = JsonPath::new();
        assert_eq!("$".to_string(), path.as_string())
    }

    #[test]
    fn a_new_rooted_path_should_not_be_empty() {
        let path = JsonPath::new();
        assert!(!path.is_empty())
    }

    #[test]
    fn simple_paths_should_have_correct_representations() {
        let path = JsonPath::new()
            .push_str_selector("a")
            .push_str_selector("b")
            .push_str_selector("c")
            .push_str_selector("d");
        assert_eq!(&path.as_string(), "$.a.b.c.d")
    }

    #[test]
    fn complex_paths_should_have_correct_representations() {
        let path = JsonPath::new()
            .push_str_selector("array")
            .push_wildcard_selector()
            .push_index_select(4)
            .push_range_selector(6, 7);
        assert_eq!(path.as_string(), "$.array.[*].[4].[6..7]")
    }

    #[test]
    fn a_root_and_partial_paths_can_be_concatenated_correctly() {
        let mut root = JsonPath::new();
        let partial = JsonPath::new_partial().push_str_selector("a");
        root = root + &partial;
        assert_eq!(root.as_string(), "$.a")
    }

    #[test]
    #[should_panic]
    fn concatenating_two_rooted_paths_should_panic() {
        let root1 = JsonPath::new();
        let root2 = JsonPath::new();
        let _combined = root1 + &root2;
    }

    #[test]
    #[should_panic]
    fn concatenating_a_root_path_to_a_partial_should_panic() {
        let partial = JsonPath::new_partial();
        let root = JsonPath::new();
        let _combined = partial + &root;
    }
}
