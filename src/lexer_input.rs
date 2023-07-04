use crate::coords::Coords;

/// Structure to manage input state information for the lexer.  Allows for an absolute position as well as a sliding
/// buffer of (as of yet) unconsumed entries
#[derive()]
pub struct LexerInput<'a> {

    /// The underlying source of characters
    chars : &'a mut dyn Iterator<Item = char>,

    /// The absolute [Coords]
    absolute_coords : Coords,

    /// Input buffer
    buffer : Vec<(char, Coords)>,

    /// Index to the front of the buffer
    front_index : usize,

    /// Index to the back of the buffer
    back_index : usize
}

impl <'a> LexerInput<'a> {
    /// Create a new state instance with all the defaults
    pub fn new(chars: &'a mut dyn Iterator<Item = char>) -> Self {
        LexerInput {
            chars,
            absolute_coords: Coords::default(),
            buffer : vec![],
            front_index: 0,
            back_index: 0
        }
    }

    /// Reset the state without resetting the state of the underlying char iterator
    pub fn reset(&mut self) {
        todo!()
    }

    /// Get the absolute position in the underlying input
    pub fn absolute(&self) -> &'a Coords {
        &self.absolute_coords
    }

    /// Get the optional [char] at the front of the buffer
    pub fn front(&self) -> Option<&(char, Coords)> {
        todo!()
    }

    /// Get the optional [char] at the back of the buffer
    pub fn back(&self) -> Option<&(char, Coords)> {
        todo!()
    }

}
