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
//! The Flat ANF form of the code is then interpreted by the `interpreter`
//! module. See [here](https://remexre.xyz/oftlisp-2018-03-12-update/) for
//! more info.

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate num;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate podio;
extern crate semver;
extern crate symbol;

pub mod anf;
pub mod ast;
mod error;
pub mod flatanf;
mod gensym;
pub mod interpreter;
pub mod intrinsics;
mod literal;
pub mod modules;
mod parser;
mod sanity;
mod util;

use std::collections::{HashMap, HashSet};

use symbol::Symbol;

pub use error::{Error, ErrorKind};
use interpreter::Value;
pub use literal::Literal;
pub use parser::{parse_file, parse_program};

/// A trait for a built-in package.
pub trait BuiltinPackage {
    /// Returns a mapping between module names (without the package part) and
    /// the names of the values the modules declare.
    fn decls() -> HashMap<Symbol, HashSet<Symbol>>;

    /// Returns the name of the package.
    fn name() -> Symbol;

    /// Returns a mapping between fully qualified names (including the package
    /// part) and values.
    fn values() -> HashMap<Symbol, Value>;
}
