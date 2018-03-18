use std::io::Error as IoError;

use symbol::Symbol;

use ast::Error as AstError;
use modules::MetadataError;

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

    /// Two different decls have the same name.
    #[fail(display = "There are two decls named {}",_0)]
    DuplicateDeclName(Symbol),

    /// Two different variables in a letrec have the same name.
    #[fail(display = "There are two variables in the same letrec named {}", _0)]
    DuplicateLetrecName(Symbol),

    /// An error from the interpreter.
    #[fail(display = "Evaluation error: {}", _0)]
    Eval(String),

    /// An error dealing with package metadata.
    #[fail(display = "{}", _0)]
    Metadata(MetadataError),

    /// A variable that was exported wasn't defined.
    #[fail(display = "{} should have exported {}, but it wasn't defined", _0, _1)]
    MissingExport(Symbol, Symbol),

    /// An expression appeared in a def that was not a literal.
    #[fail(display = "defs' expressions must be literals")]
    NonLiteralDef,

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

    /// The right-hand side of a letrec binding can't be a variable.
    #[fail(display = "The right-hand side of a letrec binding can't be a variable")]
    VarInLetrec,
}

impl From<AstError> for Error {
    fn from(err: AstError) -> Error {
        Error::Ast(err)
    }
}

impl From<MetadataError> for Error {
    fn from(err: MetadataError) -> Error {
        Error::Metadata(err)
    }
}
