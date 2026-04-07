# Lambubu
[![Rust check](https://github.com/kislball/lambubu/actions/workflows/check.yml/badge.svg)](https://github.com/kislball/lambubu/actions/workflows/check.yml)
[![Rust Clippy](https://github.com/kislball/lambubu/actions/workflows/clippy.yml/badge.svg)](https://github.com/kislball/lambubu/actions/workflows/clippy.yml)
[![Rust format](https://github.com/kislball/lambubu/actions/workflows/format.yml/badge.svg)](https://github.com/kislball/lambubu/actions/workflows/format.yml)
[![Rust tests](https://github.com/kislball/lambubu/actions/workflows/tests.yml/badge.svg)](https://github.com/kislball/lambubu/actions/workflows/tests.yml)

Lambda caclulus redex reducer.

# Installing
This project requires 2024 edition of Rust(>= 1.85). You can install the latest
version using rustup, see [here](https://rustup.rs/) for more details.

If you want to run benchmarks, you will have to use Rust >= 1.86.

NOTE: some package managers may provide outdated versions of the Rust toolchain.

## Building
```sh
$ cargo build --release
```

## Running
```sh
cargo install --path ./crates/lambubu_cli/ # install globally
cargo run # run using Cargo
```

## Usage
Lambubu CLI accepts its input via standard input. Input is a series of two kinds of statements:

1. Lambda expressions
```
(\x.x)
(,\x.x)
(λx.x)

((\a.\b.a) 2 5)
```
2. Definitions
```
TWO::(ADD 1 1)
```

Definitions are added to the context line-by-line and lambda expressions are reduced using the stretegy requested.
Each line corresponds to a single expression.

API documentation can be openned using `cargo doc --open`.

### Lambda syntax
Expression is any of the following:
1. A lambda term `(\X.Y)`, where `X` is any variable name(lowercase) and `Y` is any expression
2. A variable `a`, where `a` is a sequence of lowercase characters
3. A macro `A`, where `A` is a sequence of uppercase characters
4. Application `(A B)`, where `A` is an expression and `B` is one or more expressions.
5. Another expression in parenthesis

## Licensing
See LICENSE for details
