//! ## Another JSON Parser?
//!
//! The Chisel JSON parser aims to be a relatively simple DOM and SAX parser for JSON, that does
//! *not include* all the machinery required to support explicit serialisation from, and
//! deserialisation into `structs`/`enums` within Rust.
//!
//! It's a simple parser that is intended to allow you to choose how you want to parse a lump of *cursed* JSON,
//! and then either build/transform a DOM into a richer AST structure, or alternatively just cherry-pick the useful
//! bits of the payload via closures which are called in response to SAX parsing events.
//!
//! (*Because let's face it, JSON payloads usually come burdened with a whole load of unnecessary crap that
//! you'll never use*).
//!
#![allow(unused_imports)]
#![allow(dead_code)]

extern crate core;

use crate::coords::Span;
use std::borrow::Cow;
use std::collections::HashMap;

pub mod coords;
pub mod decoders;
pub mod dom;
pub mod errors;
pub mod events;
pub mod lexer;
pub mod paths;
pub mod sax;
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
