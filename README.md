<p align="center">
  <img src="assets/logo.png" width="50%" />
</p>

# slide

[![Build Status](https://travis-ci.com/yslide/slide.svg?branch=master)](https://travis-ci.com/yslide/slide)
[![Crates.io](https://img.shields.io/crates/v/slide)](https://crates.io/crates/slide)
[![Github help-wanted](https://img.shields.io/github/issues/yslide/slide/help%20wanted)](https://github.com/yslide/slide/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)

slide is an static expression optimizer. Given an expression like

```
x(x + 2 * 3) / (x + 6)
```

slide should be able emit the lowered expression `x`.

One of slide's design goals is compilation as a platform, where optimizations are configurable
plugins.

slide is ready for very early use. Binaries can be downloaded from the [repository
releases](https://github.com/yslide/slide/releases) or installed with `cargo`:

```
cargo install slide --version 0.0.1
# This should set slide in your path; for usage information, try
slide --help
```

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
