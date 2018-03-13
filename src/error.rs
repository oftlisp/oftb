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

    /// A variable that was exported wasn't defined.
    #[fail(display = "{} should have exported {}, but it wasn't defined", _0, _1)]
    MissingExport(Symbol, Symbol),

    /// A variable was used that doesn't exist.
    #[fail(display = "No such variable: {}", _0)]
    NoSuchVar(Symbol),

    /// A parse error.
    #[fail(display = "Syntax error in {}\n{}", _0, _1)]
    Parse(String, String),

    /// An import was made to a symbol that doesn't exist or wasn't exported.
    #[fail(display = "{} tried to import {}, but that doesn't exist (or wasn't exported)", _0, _1)]
    NonexistentImport(Symbol, Symbol),

    /// A nonexistent module was imported from.
    #[fail(display = "Nonexistent module: {}", _0)]
    NonexistentModule(Symbol),
}
