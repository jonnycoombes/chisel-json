//! Coordinate structure used to reference specific locations within parser input

use std::fmt::{Display, Formatter};

///  The main coordinate struct
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParserCoords {
    /// The absolute character position
    pub absolute: usize,
    /// The row position
    pub line: usize,
    /// The column position
    pub column: usize,
}

/// Extract the line number from a [ParserCoords]
#[macro_export]
macro_rules! line {
    ($coords : expr) => {
        $coords.line
    }
}

/// Extract the column number from a [ParserCoords]
#[macro_export]
macro_rules! column {
    ($coords : expr) => {
        $coords.column
    }
}

/// Extract the absolute number from a [ParserCoords]
#[macro_export]
macro_rules! absolute {
    ($coords : expr) => {
        $coords.absolute
    }
}

impl Default for ParserCoords {
    /// The default set of coordinates are positioned at the start of the first row
    fn default() -> Self {
        ParserCoords {
            absolute: 0,
            line: 1,
            column: 0,
        }
    }
}

impl Display for ParserCoords {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{},({})]", self.line, self.column, self.absolute)
    }
}

impl std::ops::Sub for ParserCoords {
    type Output = usize;
    /// Subtraction is based on the absolute position, could be +/-ve
    fn sub(self, rhs: Self) -> Self::Output {
        self.absolute - rhs.absolute
    }
}
