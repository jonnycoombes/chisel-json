[![Rust](https://github.com/jonnycoombes/chisel-json/actions/workflows/rust.yml/badge.svg)](https://github.com/jonnycoombes/chisel-json/actions/workflows/rust.yml)

[![crates.io](https://img.shields.io/crates/v/chisel-json.svg)](https://crates.io/crates/chisel-json)

[![crates.io](https://img.shields.io/crates/l/chisel-json)](https://crates.io/crates/chisel-json)

# {{crate}}

{{readme}}

### Crate Feature Flags

There are currently a few features defined within the crate:

| Feature | Description | Default Feature? |
|---------|-------------|---------|
| `mixed_numerics` | Should numbers be parsed separately as `i64` and `f64`? | `yerp` |
| `default_utf8_encoding` | Will the default encoding be UTF-8? | `yerp` |



### Build & Test

In order to build locally you can just use the standard `cargo build` command and associated variants,
however there is also a supplementary [Makefile.toml](./Makefile.toml) included in the source if you prefer to use
`cargo-make`.

To regenerate the README.md file as you build - you should either use:

```
cargo make
```
or alternatively,

```
cargo readme > README.md
```

There are a number of benchmarks included based on the *most excellent*
[criterion](https://github.com/bheisler/criterion.rs) within the source, which can be run using either the supplied
[benchmark.sh](./benchmark.sh) script, or alternatively by using the associated `cargo make` targets.
