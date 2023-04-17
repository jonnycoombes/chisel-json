/// Two types of parser are currently implemented:
/// - DOM: parses the supplied input and produces a complete DOM representation of the contents.
/// - SAX: parses the supplied input and generates a series of events relating to the tokens found
/// within.

/// The DOM-based parser
pub mod dom;
/// The SAX-based parser
pub mod sax;
