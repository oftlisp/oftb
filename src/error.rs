use std::io::Error as IoError;

use symbol::Symbol;

use ast::Error as AstError;

/// An error from `oftb`.
#[derive(Debug, Fail)]
pub enum Error {
    /// An error converting values to an AST.
    #[fail(display = "{}", _0)]
    Ast(AstError),

    /// An error opening a source file.
    #[fail(display = "Couldn't open `{:?}': {}", _0, _1)]
    CouldntOpenSource(String, IoError),

    /// A dependency loop was detected.
    #[fail(display = "Dependency loop involving the {} module", _0)]
    DependencyLoop(Symbol),

    /// An error from the interpreter.
    #[fail(display = "Evaluation error: {}", _0)]
    Eval(String),

    /// A parse error.
    #[fail(display = "Syntax error in {}\n{}", _0, _1)]
    Parse(String, String),

    /// A nonexistent module was imported from.
    #[fail(display = "Nonexistent module: {}", _0)]
    NonexistentModule(Symbol),
}
