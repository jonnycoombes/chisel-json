#![allow(unused_macros)]
//! Basic JSONPath generation and manipulation.  Note that this is *not* supposed to be a complete
//! implementation of RFC 8259...there is just enough JSONPath goodness within this module to
//! support what the SAX and DOM parsers do.
use std::borrow::Cow;
use std::collections::{vec_deque, VecDeque};
use std::fmt::Display;
use std::ops::Add;

/// The default separator to use within generated paths
const PATH_SEPARATOR: char = '.';

/// An enumeration fo various different path components
#[derive(Debug, Clone, PartialEq)]
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
            Self::NameSelector(s) => write!(f, "['{}']", s),
            Self::WildcardSelector => write!(f, "[*]"),
            Self::IndexSelector(i) => write!(f, "[{}]", i),
            Self::RangeSelector(i, j) => write!(f, "[{}..{}]", i, j),
        }
    }
}

/// Macro to check whether a given [JsonPathComponent] is an index selector
macro_rules! is_index_selector {
    ($comp : expr) => {
        match $comp {
            JsonPathComponent::IndexSelector(_) => true,
            _ => false,
        }
    };
}

/// Macro to check whether a given [JsonPathComponent] is a root selector
macro_rules! is_root_selector {
    ($comp : expr) => {
        match $comp {
            JsonPathComponent::Root => true,
            _ => false,
        }
    };
}

/// Macro to check whether a given [JsonPathComponent] is a wildcard root selector
macro_rules! is_wildcard_selector {
    ($comp : expr) => {
        match $comp {
            JsonPathComponent::WildcardSelector => true,
            _ => false,
        }
    };
}

/// macro to check whether a given [JsonPathComponent] is a name selector
macro_rules! is_name_selector {
    ($comp : expr) => {
        match $comp {
            JsonPathComponent::NameSelector(_) => true,
            _ => false,
        }
    };
}

/// macro to check whether a given [JsonPathComponent] is a range selector
macro_rules! is_range_selector {
    ($comp : expr) => {
        match $comp {
            JsonPathComponent::RangeSelector(_, _) => true,
            _ => false,
        }
    };
}

/// Struct for creating and manipulating Json paths vaguely compatible with a subset of RFC 8259.
/// Each instance of [JsonPath] comprises of multiple [JsonPathComponent]s
#[derive(Debug)]
pub struct JsonPath<'a> {
    /// The path components
    components: VecDeque<JsonPathComponent<'a>>,
}

impl<'a> JsonPath<'a> {
    /// Create a new, partial (no root) [JsonPath] instance
    pub fn new_partial() -> Self {
        JsonPath {
            components: VecDeque::new(),
        }
    }

    /// Create a new [JsonPath] instance with just a root component
    pub fn new() -> Self {
        JsonPath {
            components: VecDeque::from([JsonPathComponent::Root]),
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
    pub fn push_str_selector(&mut self, name: &str) {
        self.components
            .push_back(JsonPathComponent::NameSelector(Cow::Owned(String::from(
                name.replace("\"", ""),
            ))));
    }

    /// Push a new [JsonPathComponent::IndexSelector] based on a given index
    pub fn push_index_select(&mut self, index: usize) {
        self.components
            .push_back(JsonPathComponent::IndexSelector(index));
    }

    /// Push a new [JsonPathComponent::RangeSelector] based on a given start and end index
    pub fn push_range_selector(&mut self, start: usize, end: usize) {
        self.components
            .push_back(JsonPathComponent::RangeSelector(start, end));
    }

    /// Push a new [JsonPathComponent::WildcardSelector]
    pub fn push_wildcard_selector(&mut self) {
        self.components
            .push_back(JsonPathComponent::WildcardSelector);
    }

    /// Appends a new [JsonPathComponent] to the end of the path
    pub fn push(&mut self, component: JsonPathComponent<'a>) {
        self.components.push_back(component);
    }

    /// Pops the last [JsonPathComponent] from the end of the path (if it exists)
    pub fn pop(&mut self) -> Option<JsonPathComponent<'a>> {
        self.components.pop_back()
    }

    /// Checks whether a path points to an array element within the source JSON
    pub fn is_array_path(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        is_index_selector!(self.components.iter().last().unwrap())
    }

    /// Returns the number of [JsonPathComponent]s within the path
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Determines whether a given [JsonPath] matches another given [JsonPath] strictly,
    /// in that wildcards do not match against ranges within the path, and each path must
    /// be identical to each other in the sense that all components match exactly
    pub fn matches_strict(&self, rhs: &JsonPath<'a>) -> bool {
        if self.is_empty() && rhs.is_empty() {
            return true;
        }
        if self.len() != rhs.len() {
            return false;
        };
        self.components
            .iter()
            .zip(rhs.components.iter())
            .fold(true, |acc, comps| acc && comps.0 == comps.1)
    }

    /// Determines whether a path matches a given path according to the following rules:
    /// - A wildcard selector should match against an index selector in the same position
    /// - Name selectors only match based on strict equality
    /// - Indexed selectors match a range selector iff the index lies within the range supplied
    /// - Range selectors only match strictly with other range selectors (hmmmm)
    pub fn matches(&self, rhs: &JsonPath<'a>) -> bool {
        if self.is_empty() && rhs.is_empty() {
            return true;
        }
        if self.len() != rhs.len() {
            return false;
        };
        self.components
            .iter()
            .zip(rhs.components.iter())
            .fold(true, |acc, comp| acc && Self::match_path_components(comp))
    }

    /// Pattern match on pairs of [JsonPathComponent]s, and perform "special" equality checks for
    /// things like [JsonPathComponent::IndexSelector] and [JsonPathComponent::RangeSelector]. We
    /// need to make sure that we take care of commutativity in the match arms
    fn match_path_components(pair: (&JsonPathComponent, &JsonPathComponent)) -> bool {
        match pair {
            (JsonPathComponent::IndexSelector(i), JsonPathComponent::IndexSelector(j)) => i == j,
            (JsonPathComponent::IndexSelector(i), JsonPathComponent::RangeSelector(j, k)) => {
                j <= i && i <= k
            }
            (JsonPathComponent::RangeSelector(j, k), JsonPathComponent::IndexSelector(i)) => {
                j <= i && i <= k
            }
            (JsonPathComponent::WildcardSelector, JsonPathComponent::IndexSelector(_)) => true,
            (JsonPathComponent::IndexSelector(_), JsonPathComponent::WildcardSelector) => true,
            (a, b) => a == b,
        }
    }

    /// Returns the (optional) head of the path
    pub fn head(&mut self) -> Option<JsonPathComponent<'a>> {
        self.components.pop_front()
    }

    /// Returns a new [JsonPath] instance containing the tail elements of a path. If the existing
    /// path is empty, or only has a single element (head) then None is returned.
    pub fn tail(&mut self) -> Option<Self> {
        if self.is_empty() || self.len() == 1 {
            return None;
        }
        Some(JsonPath {
            components: self.components.split_off(1),
        })
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
            .for_each(|c| self.components.push_back(c.clone()));
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
        let mut path = JsonPath::new();
        path.push_str_selector("a");
        path.push_str_selector("b");
        path.push_str_selector("c");
        path.push_str_selector("d");
        assert_eq!(&path.as_string(), "$.['a'].['b'].['c'].['d']")
    }

    #[test]
    fn complex_paths_should_have_correct_representations() {
        let mut path = JsonPath::new();
        path.push_str_selector("array");
        path.push_wildcard_selector();
        path.push_index_select(4);
        path.push_range_selector(6, 7);
        assert_eq!(path.as_string(), "$.['array'].[*].[4].[6..7]")
    }

    #[test]
    fn popping_elements_should_correctly_alter_representation() {
        let mut path = JsonPath::new();
        path.push_str_selector("a");
        path.push_str_selector("b");
        path.push_str_selector("c");
        path.push_str_selector("d");
        path.pop();
        path.pop();
        assert_eq!(&path.as_string(), "$.['a'].['b']")
    }

    #[test]
    fn array_paths_should_be_identified_as_such() {
        let mut path = JsonPath::new();
        path.push_str_selector("a");
        path.push_index_select(4);
        assert!(path.is_array_path())
    }

    #[test]
    fn root_and_partial_paths_can_be_concatenated_correctly() {
        let mut root = JsonPath::new();
        let mut partial = JsonPath::new_partial();
        partial.push_str_selector("a");
        root = root + &partial;
        assert_eq!(root.as_string(), "$.['a']")
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

    #[test]
    fn empty_paths_should_strictly_match() {
        let left = JsonPath::new();
        let right = JsonPath::new();
        assert!(left.matches_strict(&right))
    }

    #[test]
    fn complex_paths_should_strictly_match() {
        let mut left = JsonPath::new();
        let mut right = JsonPath::new();
        left.push_str_selector("a");
        right.push_str_selector("a");
        left.push_index_select(6);
        right.push_index_select(6);
        left.push_range_selector(4, 5);
        right.push_range_selector(4, 5);
        assert!(left.matches_strict(&right));
        assert_eq!(left.to_string(), right.to_string())
    }

    #[test]
    fn slightly_different_complex_paths_should_not_strictly_match() {
        let mut left = JsonPath::new();
        let mut right = JsonPath::new();
        left.push_str_selector("a");
        right.push_str_selector("a");
        left.push_index_select(6);
        right.push_index_select(6);
        left.push_range_selector(4, 5);
        right.push_range_selector(3, 5);
        assert!(!left.matches_strict(&right));
        assert_ne!(left.to_string(), right.to_string())
    }

    #[test]
    fn partial_paths_should_match_strictly() {
        let mut left = JsonPath::new_partial();
        let mut right = JsonPath::new_partial();
        left.push_str_selector("a");
        right.push_str_selector("a");
        left.push_wildcard_selector();
        right.push_wildcard_selector();
        assert!(left.matches(&right));
        assert_eq!(left.to_string(), right.to_string())
    }

    #[test]
    fn indexes_should_match_on_ranges() {
        // x == y
        let mut left = JsonPath::new();
        let mut right = JsonPath::new();
        left.push_range_selector(0, 4);
        right.push_index_select(3);
        assert!(left.matches(&right));

        // y == x
        let mut left = JsonPath::new();
        let mut right = JsonPath::new();
        left.push_range_selector(0, 4);
        right.push_index_select(3);
        assert!(right.matches(&left))
    }
    #[test]
    fn indexes_should_not_match_outside_of_ranges() {
        // x == y
        let mut left = JsonPath::new();
        let mut right = JsonPath::new();
        left.push_range_selector(0, 4);
        right.push_index_select(6);
        assert!(!left.matches(&right));

        // y == x
        let mut left = JsonPath::new();
        let mut right = JsonPath::new();
        left.push_range_selector(0, 4);
        right.push_index_select(6);
        assert!(!right.matches(&left))
    }

    #[test]
    fn should_correctly_return_the_head() {
        let mut path = JsonPath::new();
        let head = path.head();
        assert!(head.is_some())
    }

    #[test]
    fn should_correctly_return_a_tail() {
        let mut path = JsonPath::new();
        path.push_str_selector("a");
        let tail = path.tail();
        assert!(tail.is_some());
        assert_eq!(tail.unwrap().len(), 1)
    }

    /// Simple example function for recursing a path
    fn recurse_path<'a>(
        acc: &'a mut Vec<JsonPathComponent<'a>>,
        path: &'a mut JsonPath,
    ) -> &'a Vec<JsonPathComponent<'a>> {
        match path.head() {
            Some(component) => {
                acc.push(component);
                recurse_path(acc, path)
            }
            None => acc,
        }
    }
    #[test]
    fn should_be_able_to_recurse_over_paths() {
        let mut path = JsonPath::new();
        path.push_str_selector("a");
        path.push_str_selector("b");
        path.push_str_selector("c");
        path.push_str_selector("d");
        path.push_str_selector("e");
        assert_eq!(path.len(), 6);

        let mut acc = vec![];
        let result = recurse_path(&mut acc, &mut path);
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn tail_should_return_the_correct_elements() {
        let mut path = JsonPath::new();
        path.push_str_selector("a");
        path.push_str_selector("b");
        path.push_str_selector("c");
        path.push_str_selector("d");
        path.push_str_selector("e");
        let tail = path.tail().unwrap().tail().unwrap();
        assert_eq!(tail.len(), 4);
    }

    #[test]
    fn recursing_over_paths_should_consume_original() {
        let mut path = JsonPath::new();
        path.push_str_selector("a");
        path.push_str_selector("b");
        path.push_str_selector("c");
        path.push_str_selector("d");
        path.push_str_selector("e");
        assert_eq!(path.len(), 6);
        let mut acc = vec![];
        let _result = recurse_path(&mut acc, &mut path);
        assert_eq!(path.len(), 0)
    }
}
