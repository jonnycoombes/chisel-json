//! Coordinate structure used to reference specific locations within parser input
#![allow(clippy::len_without_is_empty)]

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

/// A [Coord] represents a single location within the parser input
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
}

/// Extract the line number from a [Coords]
#[macro_export]
macro_rules! line {
    ($coords : expr) => {
        $coords.line
    };
}

/// Extract the column number from a [Coords]
#[macro_export]
macro_rules! column {
    ($coords : expr) => {
        $coords.column
    };
}

/// Extract the absolute number from a [Coords]
#[macro_export]
macro_rules! absolute {
    ($coords : expr) => {
        $coords.absolute
    };
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

impl Display for Coords {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(line: {}, column: {}, absolute: {})",
            self.line, self.column, self.absolute
        )
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

/// A [Span] represents a linear interval within the parser input
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
