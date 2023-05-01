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
    /// Root element of a pointer
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
                &s.replace("~", ENCODED_TILDE)
                    .replace("\"", "")
                    .replace("/", ENCODED_SLASH)
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

    /// Push a whole bunch of names onto the end of the path in order
    pub fn push_names(&mut self, names: &[&'a str]) {
        names.iter().for_each(|n| self.push_name(n.to_string()))
    }

    /// Push a whole bunch of indexes onto the end of the path in order
    pub fn push_indexes(&mut self, indexes: &[usize]) {
        indexes.iter().for_each(|i| self.push_index(*i))
    }

    /// Push a new [JsonPointerComponent::Name] onto the end of the pointer
    pub fn push_name(&mut self, name: String) {
        if self.is_empty() {
            self.components.push_back(JsonPointerComponent::Root)
        }
        self.components
            .push_back(JsonPointerComponent::Name(Cow::Owned(name)))
    }

    /// Push a new [JsonPointerComponent::Index] onto the end of the pointer
    pub fn push_index(&mut self, index: usize) {
        if self.is_empty() {
            self.components.push_back(JsonPointerComponent::Root)
        }
        self.components
            .push_back(JsonPointerComponent::Index(index))
    }

    /// Pop the last component off the back of the pointer
    pub fn pop(&mut self) -> Option<JsonPointerComponent<'a>> {
        self.components.pop_back()
    }

    /// Checks whether a path matches another path.
    pub fn matches(&self, rhs: &'a JsonPointer) -> bool {
        self.as_str() == rhs.as_str()
    }

    /// Serialise the pointer into a string representation that's compliant with RFC 6901
    pub fn as_str(&self) -> Cow<'a, str> {
        if self.is_empty() {
            return Cow::Owned("".to_string());
        }
        Cow::Owned(
            self.components
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("/"),
        )
    }
}

impl<'a> Display for JsonPointer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
mod tests {
    use super::JsonPointer;

    #[test]
    fn an_empty_pointer_should_be_represented_by_an_empty_string() {
        let s = JsonPointer::default().as_str();
        assert_eq!(s, "")
    }

    #[test]
    fn pointers_should_serialise_correctly() {
        let mut s = JsonPointer::default();
        s.push_names(&vec!["a", "b"]);
        assert_eq!("/a/b", s.as_str())
    }

    #[test]
    fn pointers_should_serialise_with_escapes_correctly() {
        let mut s = JsonPointer::default();
        s.push_names(&vec!["a/b", "c~d"]);
        s.push_index(3);
        assert_eq!("/a~1b/c~0d/3", s.as_str())
    }

    #[test]
    fn popping_should_shorten_pointers_correctly() {
        let mut s = JsonPointer::default();
        s.push_names(&vec!["a", "b", "c"]);
        assert_eq!("/a/b/c", s.as_str());
        s.pop();
        assert_eq!("/a/b", s.as_str())
    }

    #[test]
    fn popping_all_components_should_result_in_empty_pointer() {
        let mut s = JsonPointer::default();
        s.push_names(&vec!["a", "b", "c"]);
        s.pop();
        s.pop();
        s.pop();
        assert_eq!("", s.as_str())
    }

    #[test]
    fn pointers_should_serialise_indices_correctly() {
        let mut s = JsonPointer::default();
        s.push_index(0);
        s.push_index(3);
        s.push_index(2);
        assert_eq!("/0/3/2", s.as_str())
    }

    #[test]
    fn pointers_should_match() {
        let mut s = JsonPointer::default();
        let mut t = JsonPointer::default();
        s.push_name("b".to_string());
        s.push_index(9);
        t.push_name("b".to_string());
        t.push_index(9);
        assert!(s.matches(&t))
    }
}
