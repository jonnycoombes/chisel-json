use std::collections::VecDeque;
use std::path::Path;

/// Enumeration of different path elements
#[derive(Debug, Copy, Clone)]
pub enum PathElement {
    /// Root of a path
    Root,
    /// Root of an object within the path
    ObjectRoot,
    /// An array element
    ArrayElement(usize),
    /// An object key within the path. Carries a hash value for a string value that can be looked up
    ObjectKey(u64),
}

/// A stack structure for managing path elements as they are synthesised by the parser. Really just
/// a wrapper around a vec with some additional fairings to make life easier when transposing 
/// path components into strings
#[derive(Default, Clone)]
pub struct PathElementStack {
    /// Internal deque structure
    vec: Vec<PathElement>,
}

impl PathElementStack {
    
    /// Push a new [PathElement] onto the stack
    pub fn push(&mut self, element: PathElement) {
        self.vec.push(element);
    }

    /// Pop the front [PathElement] off the stack
    pub fn pop(&mut self) -> Option<PathElement> {
       self.vec.pop() 
    }

    /// Take a peek at the tos of the stack
    pub fn peek(&self) -> Option<&PathElement> {
        self.vec.first()
    }

    /// Clears the stack
    pub fn clear(&mut self) {
        self.vec.clear()
    }
    
    /// Retrieves the length of the stack
    pub fn len(&self) -> usize {
        self.vec.len()
    }
    
    /// Checks whether the stack is empty or not
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
    
}
