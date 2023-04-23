use std::io::BufRead;

use chisel_decoders::utf8::Utf8Decoder;

/// Enumeration of different supported encoding types
pub enum Encoding {
    Utf8,
}

/// A struct that is essentially a factory for creating new instances of [char] iterators,
/// based on a specified encoding type
#[derive(Default)]
pub struct DecoderSelector {}

impl DecoderSelector {
    /// Create and return an instance of the default configured byte decoder / char iterator
    pub fn default_decoder<'a, Buffer: BufRead>(
        &'a self,
        buffer: &'a mut Buffer,
    ) -> impl Iterator<Item = char> + 'a {
        Utf8Decoder::new(buffer)
    }

    /// Create and return an instance of a given byte decoder / char iterator based on a specific
    /// encoding
    pub fn new_decoder<'a, Buffer: BufRead>(
        &'a self,
        buffer: &'a mut Buffer,
        encoding: Encoding,
    ) -> impl Iterator<Item = char> + 'a {
        match encoding {
            Encoding::Utf8 => Utf8Decoder::new(buffer),
        }
    }
}
