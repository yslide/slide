<p align="center">
  <img src="assets/logo.png" width="50%" />
</p>

# slide

[![Build Status](https://travis-ci.com/yslide/slide.svg?branch=base)](https://travis-ci.com/yslide/slide)
[![Crates.io](https://img.shields.io/crates/v/slide)](https://crates.io/crates/slide)
[![Github help-wanted](https://img.shields.io/github/issues/yslide/slide/help%20wanted)](https://github.com/yslide/slide/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)

slide is an expression rewrite system and validator. Given an expression like

```math
x(x + 2 * 3) / (x + 6)
```

slide should be able emit the simplified expression `x`.

slide's design goals include

- Simplification as a platform, where rewrite rules are user-configurable.
    For example, you should be able to give to (or remove from!) slide a rule like `x^2 dx -> 2x`,
    and slide will incorporate the rule in reducing an expression.
    This can be thought of analogously to tunable optimizations in a compiler.
- Support for interactive user interfaces, including text editor features for
    documents like Tex in text editors. More information on this is described
    [below](#editor-support).
- Validation of statement correctness.

## Usage

slide is ready for very early use. The easiest way to try out slide is via our
[web UI](https://yslide.github.io).

Binaries can be downloaded from the [repository
releases](https://github.com/yslide/slide/releases) or installed with `cargo`:

```
cargo install slide --version 0.0.1
# This should set slide in your path; for usage information, try
slide --help
```

### Editor Support

slide has a language server and supports additional integration with some text editors,
providing analysis and simplification of mathematical expressions in documents.
For more information, see the [editor](./editor) directory.

## Contributing

Lots of features are still incomplete; please
[file an issue](https://github.com/yslide/slide/issues/new) when you see something that could be
improved. This is one of the best (and easiest!) ways to contribute to slide.

All contributions are warmly welcomed. For more information, including how to contribute to the
development of slide, see our [contribution docs](./CONTRIBUTING.md).

## libslide

The slide project exposes its library publicly, providing an API you can embed in your other Rust
apps. To add `libslide` to your project, ensure the following in your `Cargo.toml`:

```
libslide = "0.0.1"
```
