# oftb

[![Build Status](https://travis-ci.org/oftlisp/oftb.svg?branch=master)](https://travis-ci.org/oftlisp/oftb)
[![Dependency Status](https://deps.rs/repo/github/oftlisp/oftb/status.svg)](https://deps.rs/repo/github/oftlisp/oftb)

The OftLisp bootstrapper.

Requires Rust version 1.26.0 or later.

## Bootstrapping Process

Note that `build.py` performs all of these steps.

### Stage 0: `oftb`

First, `oftb` is built.
This is a potentially time-consuming process; Rust code takes a long time to build.
You can skip this step (if you've already run it) by passing `--no-oftb-build` to `build.py`.

`oftb` performs bytecode compilation to `ofta` files, and interprets `ofta` files.
It does not (currently) include a garbage collector.

### Stage 0.5: Generate `ministd/prelude` and `macro-expander/interpreter/env`

Since these two modules both rely on every export from the prelude (and are therefore a pain to update), they're generated.
To ensure that up-to-date versions are used, the generation process runs every time `build.py` is run.

### Stage 1: `macro-expander`

`macro-expander` is an interpreter that performs macro expansion.

### Stage 2: `oftc` bootstrap

### Stage 3: `oftc`

## Subprojects

### macro-expander

There's also a macro expanding interpreter written in OftLisp here, for bootstrapping.

### ministd

This repo also contains ministd, the trimmed-down version of the standard library used when bootstrapping.
Notably, ministd does not provide any macros.
Functions that normally exist as a variadic macro version and a fixed-arity function version are usually defined with a fixed arity.

## License

Licensed under either of

 * Apache License, Version 2.0, (http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
