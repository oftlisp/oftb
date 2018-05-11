# oftb

[![Build Status](https://travis-ci.org/oftlisp/oftb.svg?branch=master)](https://travis-ci.org/oftlisp/oftb)
[![Dependency Status](https://deps.rs/repo/github/oftlisp/oftb/status.svg)](https://deps.rs/repo/github/oftlisp/oftb)

The OftLisp bootstrapper.

Requires Rust version 1.26.0 or later.

## oftb-macro-expander

There's also a macro expander written in OftLisp here, for bootstrapping.

## ministd

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
