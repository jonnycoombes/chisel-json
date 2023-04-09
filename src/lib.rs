#![allow(unused_imports)]
#![allow(dead_code)]

extern crate core;

use crate::coords::Span;
use std::collections::HashMap;

pub mod coords;
pub mod errors;
mod events;
mod lexer;
pub mod parser;
mod paths;
mod scanner;
#[cfg(test)]
mod test_macros;

/// Basic enumeration of different Json values
pub enum JsonValue<'a> {
    /// Map of values
    Object(HashMap<&'a str, JsonValue<'a>>),
    /// Array of values
    Array(Vec<JsonValue<'a>>),
    /// Canonical string value
    String(&'a str),
    /// Canonical number value
    Number(f64),
    /// Canonical boolean value
    Boolean(bool),
    /// Canonical null value
    Null,
}

/// Wrapper struct that contains both a parsed [JsonValue] along with additional parse information
/// such as the [Span] relating to the value, and an arbitrary attribute type [T]
pub struct AttributeJsonValue<'a, T> {
    /// The wrapped [JsonValue]
    inner: JsonValue<'a>,
    /// A [Span] containing coordinates for the value
    span: Span,
    /// Attributes of type [T]
    attributes: T,
}
