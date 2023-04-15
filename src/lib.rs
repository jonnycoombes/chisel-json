#![allow(unused_imports)]
#![allow(dead_code)]

extern crate core;

use crate::coords::Span;
use std::borrow::Cow;
use std::collections::HashMap;

pub mod coords;
pub mod errors;
mod events;
pub mod lexer;
pub mod parser;
mod paths;
#[cfg(test)]
mod test_macros;

/// Basic enumeration of different Json values
#[derive(Debug)]
pub enum JsonValue<'a> {
    /// Map of values
    Object(Vec<(String, JsonValue<'a>)>),
    /// Array of values
    Array(Vec<JsonValue<'a>>),
    /// Canonical string value
    String(Cow<'a, str>),

    /// Floating point numeric value
    Float(f64),

    /// Integer numeric value
    Integer(i64),
    /// Canonical boolean value
    Boolean(bool),
    /// Canonical null value
    Null,
}
