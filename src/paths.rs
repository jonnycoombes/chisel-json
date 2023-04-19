use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt::Display;

/// An enumeration fo various different path components
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
            Self::RangeSelector(i,j) => write!(f, "[{}..{}]", i, j)
        }
    }
}

/// Struct for creating and manipulating Json paths vaguely compatible with a subset of RFC 8259
pub struct JsonPath {}
