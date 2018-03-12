//! The OftLisp bootstrapping interpreter.
//!
//! To facilitate fast interpretation, uses a bytecode compilation process:
//!
//! ```text
//! +------+    +------+    +---+    +---+    +--------+
//! |Source|--->|Values|--->|AST|--->|ANF|--->|Flat ANF|
//! +------+ ^  +------+ ^  +---+ ^  +---+ ^  +--------+
//!          |           |        |        |
//!  parser--+           +-----+  |        +--flatanf::Program::flatten
//!                            |  |
//!  ast::Module::from_values--+  +---anf::Module::convert
//! ```
//!
//! The Flat ANF form of the code is then interpreted by the `cesk` module.

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate symbol;

pub mod ast;
mod error;
mod heap;
mod parser;
mod util;

pub use error::Error;
pub use heap::{Heap, HeapRef};
pub use parser::parse_file;
