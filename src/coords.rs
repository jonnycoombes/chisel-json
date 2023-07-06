//! Coordinate structure used to reference specific locations within parser input
#![allow(clippy::len_without_is_empty)]

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

/// A [Coords] represents a single location within the parser input
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Coords {
    /// The absolute character position
    pub absolute: usize,
    /// The row position
    pub line: usize,
    /// The column position
    pub column: usize,
}

impl Coords {

}

impl Display for Coords {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[abs: {}, line: {}, column: {}]",
            self.absolute, self.line, self.column
        )
    }
}

impl Default for Coords {
    /// The default set of coordinates are positioned at the start of the first row
    fn default() -> Self {
        Coords {
            absolute: 0,
            line: 0,
            column: 0,
        }
    }
}

impl Eq for Coords {}

impl PartialOrd<Self> for Coords {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.absolute.cmp(&other.absolute) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Equal => Some(Ordering::Equal),
            Ordering::Greater => Some(Ordering::Greater),
        }
    }
}

impl Ord for Coords {
    fn cmp(&self, other: &Self) -> Ordering {
        self.absolute.cmp(&other.absolute)
    }
}

/// A [Span] represents a linear interval within the parser input, between to different [Coords]
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct Span {
    /// Start [Coords] for the span
    pub start: Coords,
    /// End [Coords] for the span
    pub end: Coords,
}

impl Span {
    /// Get the length of the span, minimum is 1
    pub fn len(&self) -> usize {
        match self.start.cmp(&self.end) {
            Ordering::Less => self.end - self.start,
            Ordering::Equal => 1,
            Ordering::Greater => self.start - self.end,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "start: {}, end: {}, length: {}",
            self.start,
            self.end,
            self.len()
        )
    }
}
