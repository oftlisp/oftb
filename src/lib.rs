//! The OftLisp bootstrapping interpreter.
//!
//! To facilitate fast interpretation, uses a bytecode compilation process:
//!
//! ```text
//! +------+    +------+    +---+    +---+    +--------+
//! |Source|--->|Values|--->|AST|--->|ANF|--->|Flat ANF|
//! +------+ ^  +------+ ^  +---+ ^  +---+ ^  +--------+
//!          |           |        |        |
//!  parser--+           +-----+  |        +--flatanf::Program::from_modules
//!                            |  |
//!  ast::Module::from_values--+  +---anf::Module::from
//! ```
//!
//! The Flat ANF form of the code is then interpreted by the `cesk` module.
//! See [here](https://remexre.xyz/oftlisp-2018-03-12-update/) for more info.

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate symbol;

pub mod anf;
pub mod ast;
mod error;
pub mod flatanf;
mod gensym;
mod heap;
mod literal;
mod parser;
mod util;

pub use error::Error;
pub use literal::Literal;
pub use parser::parse_file;
