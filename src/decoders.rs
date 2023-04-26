//! Both the DOM and SAX parser implementations operate over a stream of `char`s produced by some
//! flavour of iterator. By default, this iterator is based on a decoder that will take a stream of
//! bytes from an underlying source, and convert into a stream of `char`s.
//!
//! The [DecoderSelector] implemented within this module is used to instantiate new `char`
//! iterators, based on different encodings. (Currently only ASCII and UTF-8 are supported).
use chisel_decoders::{ascii::AsciiDecoder, utf8::Utf8Decoder};
use std::io::BufRead;

/// Enumeration of different supported encoding types
#[derive(Copy, Clone)]
pub enum Encoding {
    Utf8,
    Ascii,
}

impl Default for Encoding {
    fn default() -> Self {
        Self::Utf8
    }
}

/// A struct that is essentially a factory for creating new instances of [char] iterators,
/// based on a specified encoding type
#[derive(Default)]
pub(crate) struct DecoderSelector {}

impl DecoderSelector {
    /// Create and return an instance of the default byte decoder / char iterator. (Utf-8)
    pub fn default_decoder<'a, Buffer: BufRead>(
        &'a self,
        buffer: &'a mut Buffer,
    ) -> Box<dyn Iterator<Item = char> + 'a> {
        Box::new(Utf8Decoder::new(buffer))
    }

    /// Create and return an instance of a given byte decoder / char iterator based on a specific
    /// encoding
    pub fn new_decoder<'a, Buffer: BufRead>(
        &'a self,
        buffer: &'a mut Buffer,
        encoding: Encoding,
    ) -> Box<dyn Iterator<Item = char> + 'a> {
        match encoding {
            Encoding::Ascii => Box::new(AsciiDecoder::new(buffer)),
            Encoding::Utf8 => Box::new(Utf8Decoder::new(buffer)),
        }
    }
}
