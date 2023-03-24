//! Coordinate structure used to reference specific locations within parser input

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

impl std::ops::Sub for Coords {
    type Output = usize;
    /// Subtraction is based on the absolute position, could be +/-ve
    fn sub(self, rhs: Self) -> Self::Output {
        self.absolute - rhs.absolute
    }
}

/// A [Span] represents a linear interval within the parser input
pub struct Span {
    /// Start [Coords] for the span
    start: Coords,
    /// End [Coords] for the span
    end: Coords,
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "start: {}, end: {}, length: {}",
            self.start,
            self.end,
            self.end - self.start
        )
    }
}
