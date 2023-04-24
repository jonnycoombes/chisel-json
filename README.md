[![Rust](https://github.com/jonnycoombes/chisel-json/actions/workflows/rust.yml/badge.svg)](https://github.com/jonnycoombes/chisel-json/actions/workflows/rust.yml)

[![crates.io](https://img.shields.io/crates/v/chisel-json.svg)](https://crates.io/crates/chisel-json)

[![crates.io](https://img.shields.io/crates/l/chisel-json)](https://crates.io/crates/chisel-json)

# chisel-json

### Another JSON Parser?

The Chisel JSON parser aims to be a relatively simple DOM and SAX parser for JSON, that does
*not include* all the machinery required to support explicit serialisation from, and
deserialisation into `structs`/`enums` within Rust.

It's a bare-bones parser that is intended to allow you to choose how you want to parse a lump of *cursed* JSON,
and then either build/transform a DOM into a richer AST structure, or alternatively just cherry-pick the useful
bits of the payload via closures which are called in response to SAX parsing events.

(Because *let's face it*, JSON payloads usually come burdened with a whole load of unnecessary crap that
you'll never use).


Coming soon to a small wooden hut near you...
