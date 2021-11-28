# Dotweb

A simple web interface for Graphviz' dot.

See it running at https://dot.sourcetagsandcodes.com

## Build

Dotweb is written in [Rust] 2021 Edition, using Rust 1.56. [Cargo] is used to
build Dotweb and manage dependencies. You can install Rust and Cargo using
[`rustup`]. If you're new to Rust, [The Rust Programming Language][book] - an
introductory book about Rust - is available free online.

[Rust]: https://www.rust-lang.org/
[`rustup`]: https://rustup.rs
[Cargo]: https://doc.rust-lang.org/stable/cargo/
[book]: https://doc.rust-lang.org/book/2018-edition/index.html

Dotweb can then be built for development with:

    cargo build

The binary will be written to `target/debug/dotweb`.

Building for release can be done with:

    cargo build --release

The binary will be written to `target/release/dotweb`.

## Run

Dotweb can be built and run for development with:

    cargo run

Or the binary (once built with `cargo build`) can be run directly with:

    target/debug/dotweb

You can check Dotweb is running ok by visiting http://localhost:8080/status

Options can be provided like:

    cargo run -- --port 1234

or

    target/debug/dotweb --port 1234

## Config

### Command Line Arguments

    USAGE:
        dotweb [FLAGS] [OPTIONS]

    FLAGS:
        -h, --help       Prints help information
        -q, --quiet      Silence all output
        -V, --version    Prints version information
        -v, --verbose    Verbose mode, multiples increase the verbosity

    OPTIONS:
        -H, --host <HOST>    Host to listen on [default: 127.0.0.1]
        -P, --port <PORT>    Port to listen on [default: 8080]

## Troubleshooting

Run Dotweb at maximum output verbosity with the `-vvvv` flag, like:

    dotweb -vvvv

or

    cargo run -- -vvvv
