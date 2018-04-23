use std::fmt::{Display, Formatter, Result as FmtResult};

use failure::{Backtrace, Context, Fail};
use semver::{ReqParseError, SemVerError};
use symbol::Symbol;

use literal::Literal;

/// An error from `oftb`.
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.inner.get_context().clone()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.inner, f)
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}

/// The kind of an error from `oftb`.
#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    /// An error opening a source file.
    #[fail(display = "Couldn't open `{}'", _0)]
    CouldntOpenSource(String),

    /// An error opening a source file.
    #[fail(display = "Couldn't open directory {} in package `{}'", _0, _1)]
    CouldntReadPackageDir(String, Symbol),

    /// A dependency loop was detected between modules.
    #[fail(display = "Dependency loop involving the {} module", _0)]
    DependencyLoopInModule(Symbol),

    /// A dependency loop was detected between packages.
    #[fail(display = "Dependency loop involving the {} package", _0)]
    DependencyLoopInPackage(Symbol),

    /// It's impossible to depend on a package that doesn't export a library.
    #[fail(display = "The `{}' package must export a library to be depended on.", _0)]
    DependencyMustExportLib(Symbol),

    /// Two different decls have the same name.
    #[fail(display = "There are two decls named {}", _0)]
    DuplicateDeclName(Symbol),

    /// Two different variables in a letrec have the same name.
    #[fail(display = "There are two variables in the same letrec named {}",
           _0)]
    DuplicateLetrecName(Symbol),

    /// A field was duplicated.
    #[fail(display = "Duplicate field: {}", _0)]
    DuplicateField(Symbol),

    /// An error from the interpreter.
    #[fail(display = "Evaluation error: {}", _0)]
    Eval(String),

    /// A given dependency version was invalid.
    #[fail(display = "Bad dependency version: {}", _0)]
    IllegalDependencyVersion(ReqParseError),

    /// A given package version number was invalid.
    #[fail(display = "Bad package version: {}", _0)]
    IllegalPackageVersion(SemVerError),

    /// The given value is not a valid declaration, but appeared in a
    /// declaration context.
    #[fail(display = "Invalid declaration: {}", _0)]
    InvalidDecl(Literal),

    /// The given value is not a valid expression, but appeared in an
    /// expression context.
    #[fail(display = "Invalid expression: {}", _0)]
    InvalidExpr(Literal),

    /// A mismatch between expected and found module names.
    #[fail(display = "Expected a module named `{}', found `{}'.", _0, _1)]
    MisnamedModule(Symbol, Symbol),

    /// A mismatch between expected and found package names.
    #[fail(display = "Expected a package named `{}', found `{}'.", _0, _1)]
    MisnamedPackage(Symbol, Symbol),

    /// A variable that was exported wasn't defined.
    #[fail(display = "{} should have exported {}, but it wasn't defined", _0,
           _1)]
    MissingExport(Symbol, Symbol),

    /// A required field was missing.
    #[fail(display = "Missing field: {}", _0)]
    MissingField(Symbol),

    /// A module didn't have a module form.
    #[fail(display = "No module form was found.")]
    NoModuleForm,

    /// An expression appeared in a def that was not a literal.
    #[fail(display = "defs' expressions must be literals")]
    NonLiteralDef,

    /// A binary doesn't exist (or the package containing it wasn't loaded).
    #[fail(display = "No such binary `{}' (from the `{}' package)", _1, _0)]
    NoSuchBinary(Symbol, String),

    /// A variable was used that doesn't exist.
    #[fail(display = "No such variable: {}", _0)]
    NoSuchVar(Symbol),

    /// An import was made to a symbol that doesn't exist or wasn't exported.
    #[fail(display = "{} tried to import {}, but that doesn't exist (or wasn't exported)", _0, _1)]
    NonexistentImport(Symbol, Symbol),

    /// A nonexistent module was imported from.
    #[fail(display = "Nonexistent module: {}", _0)]
    NonexistentModule(Symbol),

    /// A parse error.
    #[fail(display = "Syntax error in {}", _0)]
    Parse(String),

    /// A value with an unexpected type was found.
    #[fail(display = "Expected {}, found {}", _0, _1)]
    Unexpected(&'static str, Literal),

    /// The right-hand side of a letrec binding can't be a variable.
    #[fail(display = "The right-hand side of the letrec binding for {} can't be a variable {} which is defined by the letrec", _0, _1)]
    VarInLetrec(Symbol, Symbol),
}
