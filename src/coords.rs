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
    /// Increment a coordinate by 1 character in the input. Optional, bump the current line number
    /// and reset the column coordinate to 1
    #[inline]
    pub fn inc(&mut self, newline: bool) {
        if newline {
            self.absolute += 1;
            self.line += 1;
            self.column = 1;
        } else {
            self.absolute += 1;
            self.column += 1;
        }
    }

    /// Increment a coordinate by n.  Does not allow for crossing newline boundaries
    #[inline]
    pub fn inc_n(&mut self, n: usize) {
        self.absolute += n;
        self.column += n;
    }
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
            line: 1,
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

impl std::ops::Sub for Coords {
    type Output = usize;
    /// Subtraction is based on the absolute position, could be +/-ve
    fn sub(self, rhs: Self) -> Self::Output {
        self.absolute - rhs.absolute
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
